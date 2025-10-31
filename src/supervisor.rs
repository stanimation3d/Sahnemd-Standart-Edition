// src/supervisor.rs (GÜNCELLENMİŞ)

use core::slice;
use crate::syscalls::{self, sys_const, TaskId};
use crate::service::{Service, ServiceState, RestartPolicy};
use crate::deps::{DependencyService, check_dependencies};

// spawn_task fonksiyonu (Değişmedi)

/// Tüm hizmetleri yöneten ana init döngüsünü başlatır ve görev sonlanmalarını izler.
pub fn run_supervisor_loop(services: &mut [DependencyService]) -> ! {
    
    println!("[SUP] Sahnemd Süpervizörü Başlatıldı.");
    let mut exit_status: u64 = 0;

    // Ana Döngü
    loop {
        let current_time = syscalls::get_system_time(); // Mevcut zamanı al
        let mut started_new = false;
        let mut should_sleep = true;
        
        // A. Bağımlılıkları Kontrol Et ve Yeni/Yeniden Başlatılacak Hizmetleri Başlat
        for dep_service in services.iter_mut() {
            let svc = &mut dep_service.service;
            
            if svc.state == ServiceState::Stopped || svc.state == ServiceState::Waiting {
                
                // 1. Önce Geri Çekilme Zamanını Kontrol Et
                if svc.next_start_time > current_time {
                    should_sleep = false; // Aktif olarak bir şey bekliyoruz, sadece yield yap
                    continue; 
                }

                // 2. Bağımlılıkları Kontrol Et
                if check_dependencies(dep_service, services) {
                    println!("[SUP] Başlatılıyor: {}", svc.name);
                    
                    // Başlatmadan önce sayacı sıfırla (Başlatma denemesi başarılı oldu)
                    svc.reset_restart_count();

                    match spawn_task(svc) {
                        Ok(task_id) => {
                            println!(" -> Başarılı. Task ID: {}", task_id);
                            svc.state = ServiceState::Running(task_id);
                            started_new = true;
                        },
                        Err(e) => {
                            eprintln!(" -> Başarısız! Hata Kodu: {}", e);
                            svc.state = ServiceState::Failed; // Anlık başarısızlık durumunda da bekleme ayarlamalıyız.
                            svc.set_backoff(current_time);
                        }
                    }
                } else {
                    svc.state = ServiceState::Waiting;
                    should_sleep = false; // Beklemede olanlar varsa da uyumamalıyız.
                }
            }
        }
        
        // B. Kapatma Kontrolü (Değişmedi)
        let active_tasks = services.iter().any(|s| matches!(s.service.state, ServiceState::Running(_)));
        let runnable_tasks = services.iter().any(|s| matches!(s.service.state, ServiceState::Stopped | ServiceState::Waiting));
        let waiting_for_backoff = services.iter().any(|s| s.service.next_start_time > current_time);

        if !active_tasks && !runnable_tasks && !waiting_for_backoff {
            println!("\n[SUP] Tüm kullanıcı alanı görevleri sonlandı. Init sistemi sonlanıyor.");
            break; 
        }

        // C. TASK_WAIT veya TASK_YIELD/SLEEP
        if active_tasks {
            // Aktif görevler varsa, bloklayarak bir görev sonlanmasını bekle.
            let wait_result = syscalls::task_wait(None, &mut exit_status as *mut u64);
            
            match wait_result {
                Ok(terminated_task_id) => {
                    println!(
                        "[SUP] GÖREV SONLANDI: ID: {} | Çıkış Durumu: {}", 
                        terminated_task_id, exit_status
                    );
                    
                    if let Some(dep_service) = services.iter_mut().find(|s| {
                        if let ServiceState::Running(id) = s.service.state {
                            id == terminated_task_id
                        } else {
                            false
                        }
                    }) {
                        let svc = &mut dep_service.service;

                        // Yeniden başlatma politikasına göre hareket et
                        let should_restart = match svc.restart_policy {
                            RestartPolicy::Always => true,
                            RestartPolicy::OnFailure => exit_status != 0,
                            RestartPolicy::Never => false,
                        };

                        if should_restart {
                            println!("[SUP] ({}) Politikası: Yeniden başlatılıyor. Geri çekilme ayarlanıyor...", svc.name);
                            svc.set_backoff(syscalls::get_system_time()); // Geri çekilme süresini ayarla
                            svc.state = ServiceState::Stopped; 
                        } else {
                            println!("[SUP] ({}) Politikası: Başarılı sonlandı/Asla. Yeniden başlatılmıyor.", svc.name);
                            svc.state = ServiceState::Failed; // Sonlandı, artık Failed olarak işaretle
                        }
                        
                    } else {
                        eprintln!("[SUP] UYARI: İzlenmeyen bir görev sonlandı: {}", terminated_task_id);
                    }
                },
                Err(e) => {
                    eprintln!("[SUP] SYSCALL_TASK_WAIT Hatası: {}", e);
                    syscalls::task_yield();
                }
            }
        } else if waiting_for_backoff {
            // Çalışan görev yok ama bir hizmet geri çekilme süresi bekliyor.
            // En kısa bekleme süresini bul ve o kadar uyuyalım.
            let min_delay = services.iter()
                .filter(|s| s.service.next_start_time > current_time)
                .map(|s| s.service.next_start_time.saturating_sub(current_time))
                .min().unwrap_or(0);

            if min_delay > 0 {
                println!("[SUP] Geri çekilme bekleniyor... {}ms uyku.", min_delay);
                // Uyku fonksiyonunu kullan, bu da init sisteminin dinlenmesini sağlar.
                syscalls::task_sleep(min_delay); 
            } else {
                // Eğer min_delay 0'a yakınsa veya bir hata varsa yield yap.
                syscalls::task_yield();
            }
        } else {
            // Aktif görev yok ve geri çekilme de beklenmiyor. Sadece CPU'yu devret.
            syscalls::task_yield();
        }
    }
    
    task_exit(0); 
}
