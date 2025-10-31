// src/supervisor.rs (GÜNCELLENMİŞ)

use core::slice;
use crate::syscalls::{self, sys_const, TaskId};
use crate::service::{Service, ServiceState, RestartPolicy}; // RestartPolicy Eklendi
use crate::deps::{DependencyService, check_dependencies};

// spawn_task fonksiyonu değişmedi

/// Tüm hizmetleri yöneten ana init döngüsünü başlatır ve görev sonlanmalarını izler.
pub fn run_supervisor_loop(services: &mut [DependencyService]) -> ! {
    
    println!("[SUP] Sahnemd Süpervizörü Başlatıldı.");
    let mut exit_status: u64 = 0;

    // Ana Döngü
    loop {
        let mut started_new = false;
        
        // A. Bağımlılıkları Kontrol Et ve Yeni/Yeniden Başlatılacak Hizmetleri Başlat
        for dep_service in services.iter_mut() {
            if dep_service.service.state == ServiceState::Stopped || dep_service.service.state == ServiceState::Waiting {
                
                if check_dependencies(dep_service, services) {
                    println!("[SUP] Başlatılıyor: {}", dep_service.service.name);
                    match spawn_task(&dep_service.service) {
                        Ok(task_id) => {
                            println!(" -> Başarılı. Task ID: {}", task_id);
                            dep_service.service.state = ServiceState::Running(task_id);
                            started_new = true;
                        },
                        Err(e) => {
                            eprintln!(" -> Başarısız! Hata Kodu: {}", e);
                            dep_service.service.state = ServiceState::Failed;
                        }
                    }
                } else {
                    dep_service.service.state = ServiceState::Waiting;
                }
            }
        }
        
        // B. Aktif Görev Kalmadıysa ve Başlatılacak Başka Bir Şey Yoksa Kapat
        let active_tasks = services.iter().any(|s| matches!(s.service.state, ServiceState::Running(_)));
        let runnable_tasks = services.iter().any(|s| matches!(s.service.state, ServiceState::Stopped | ServiceState::Waiting));

        if !active_tasks && !runnable_tasks {
            println!("\n[SUP] Tüm kullanıcı alanı görevleri sonlandı. Init sistemi sonlanıyor.");
            break; 
        }

        // C. SYSCALL_TASK_WAIT ile Çocuk Görevlerin Sonlanmasını Bekle
        if active_tasks {
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
                        // --- YENİ: YENİDEN BAŞLATMA MANTIĞI ---
                        match dep_service.service.restart_policy {
                            RestartPolicy::Always => {
                                println!("[SUP] ({}) Politikası: Yeniden başlatılıyor (Stopped yapılıyor).", dep_service.service.name);
                                dep_service.service.state = ServiceState::Stopped; // Bir sonraki döngüde tekrar başlatılacak
                            }
                            RestartPolicy::OnFailure => {
                                if exit_status != 0 {
                                    println!("[SUP] ({}) Politikası: Hata (exit: {}) nedeniyle yeniden başlatılıyor.", dep_service.service.name, exit_status);
                                    dep_service.service.state = ServiceState::Stopped;
                                } else {
                                    println!("[SUP] ({}) Politikası: Başarılı sonlandı. Yeniden başlatılmıyor (Failed yapılıyor).", dep_service.service.name);
                                    dep_service.service.state = ServiceState::Failed;
                                }
                            }
                            RestartPolicy::Never => {
                                println!("[SUP] ({}) Politikası: Yeniden başlatılmıyor (Failed yapılıyor).", dep_service.service.name);
                                dep_service.service.state = ServiceState::Failed;
                            }
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
        } else {
            // Aktif görev yok ama başlatılabilir görevler var (bağımlılık bekliyorlar).
            syscalls::task_yield();
        }
    }
    
    task_exit(0); 
}
