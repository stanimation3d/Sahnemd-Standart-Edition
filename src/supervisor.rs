// src/supervisor.rs (GÜNCELLENMİŞ)

use core::slice;
use crate::syscalls::{self, sys_const, TaskId};
use crate::service::{Service, ServiceState, RestartPolicy};
use crate::deps::{DependencyService, check_dependencies};
use crate::ipc::{ControlMessage, ChannelHandle, channel_receive_non_blocking}; // YENİ

// spawn_task fonksiyonu (Değişmedi)

// ... spawn_task fonksiyonu ...

/// Tüm hizmetleri yöneten ana init döngüsünü başlatır ve görev sonlanmalarını izler.
pub fn run_supervisor_loop(services: &mut [DependencyService], control_channel: ChannelHandle) -> ! {
    
    println!("[SUP] Sahnemd Süpervizörü Çalışıyor. Kontrol Kanalı Dinleniyor...");
    let mut exit_status: u64 = 0;

    // Ana Döngü
    loop {
        let current_time = syscalls::get_system_time(); 
        
        // --- 1. Kontrol Kanalını Dinle (Non-Blocking) ---
        match channel_receive_non_blocking(control_channel) {
            Ok(message) => {
                println!("[SUP] Kontrol Mesajı Alındı: {:?}", message);
                match message {
                    ControlMessage::Shutdown => {
                        println!("[SUP] Kapatma komutu alındı. Döngüden çıkılıyor.");
                        break; // Döngüyü kır, sistem kapanışına git

                        for dep_service in services.iter_mut() {
                            if let ServiceState::Running(id) = dep_service.service.state {
                                println!("[SUP] -> Görev sonlandırılıyor: {} ({})", dep_service.service.name, id);
                            task_kill(id);
                            }
                        }
                    }
                    ControlMessage::RestartService(target_id) => {
                        // Basitçe hizmeti durdurulmuş (Stopped) duruma getir. Yeniden başlatma mantığı onu tekrar başlatacaktır.
                        if let Some(dep_service) = services.iter_mut().find(|s| {
                            if let ServiceState::Running(id) = s.service.state { id == target_id } else { false }
                        }) {
                            println!("[SUP] Hizmet yeniden başlatma isteği: {}", dep_service.service.name)
                            let result = task_kill(target_id);
                            if result == 0 {
                                // Durumu Stopped yaparsak, yeniden başlatma mantığı hemen yakalar ve backoff ayarlar.
                                // task_kill başarılıysa, TASK_WAIT bir sonraki adımda yakalar.
                            } else {
                                eprintln!("[SUP] Hata: Görev sonlandırılamadı. Hata Kodu: {}", result);
                            }
                        } else {
                            eprintln!("[SUP] Hata: Yeniden başlatılmak istenen görev bulunamadı: {}", target_id);
                        }
                    }
                    _ => eprintln!("[SUP] Bilinmeyen/İşlenmeyen Kontrol Mesajı: {:?}", message),
                }
            },
            Err(0) => {
                // Veri yok (Non-blocking çağrı başarılı)
            }, 
            Err(e) => {
                eprintln!("[SUP] IPC Hata Kodu: {}", e);
            }
        }

        // --- 2. Hizmet Yönetimi (Başlatma/Backoff) ---
    for dep_service in services.iter_mut() {
            let svc = &mut dep_service.service;
            
            if svc.state == ServiceState::Stopped || svc.state == ServiceState::Waiting {
                
                if svc.next_start_time > current_time {
                    continue; 
                }

                if check_dependencies(dep_service, services) {
                    println!("[SUP] Başlatılıyor: {}", svc.name);
                    
                    svc.reset_restart_count();

                    match spawn_task(svc) {
                        Ok(task_id) => {
                            println!(" -> Başarılı. Task ID: {}", task_id);
                            svc.state = ServiceState::Running(task_id);
                        },
                        Err(e) => {
                            eprintln!(" -> Başarısız! Hata Kodu: {}", e);
                            svc.state = ServiceState::Failed; 
                            svc.set_backoff(current_time);
                        }
                    }
                } else {
                    svc.state = ServiceState::Waiting;
                }
            }
        }
        
        // B. Kapatma Kontrolü
        let active_tasks = services.iter().any(|s| matches!(s.service.state, ServiceState::Running(_)));
        let runnable_tasks = services.iter().any(|s| matches!(s.service.state, ServiceState::Stopped | ServiceState::Waiting));
        let waiting_for_backoff = services.iter().any(|s| s.service.next_start_time > current_time);

        if !active_tasks && !runnable_tasks && !waiting_for_backoff {
            println!("\n[SUP] Tüm kullanıcı alanı görevleri sonlandı. Init sistemi sonlanıyor.");
            break; 
        }

        // C. TASK_WAIT veya TASK_SLEEP
        if active_tasks {
            let wait_result = syscalls::task_wait(None, &mut exit_status as *mut u64);
            
            match wait_result {
                Ok(terminated_task_id) => {
                    println!(
                        "[SUP] GÖREV SONLANDI: ID: {} | Çıkış Durumu: {}", 
                        terminated_task_id, exit_status
                    );
                    
                    if let Some(dep_service) = services.iter_mut().find(|s| {
                        if let ServiceState::Running(id) = s.service.state { id == terminated_task_id } else { false }
                    }) {
                        let svc = &mut dep_service.service;

                        let should_restart = match svc.restart_policy {
                            RestartPolicy::Always => true,
                            RestartPolicy::OnFailure => exit_status != 0,
                            RestartPolicy::Never => false,
                        };

                        if should_restart {
                            println!("[SUP] ({}) Politikası: Yeniden başlatılıyor. Geri çekilme ayarlanıyor...", svc.name);
                            svc.set_backoff(syscalls::get_system_time()); 
                            svc.state = ServiceState::Stopped; 
                        } else {
                            println!("[SUP] ({}) Politikası: Yeniden başlatılmıyor.", svc.name);
                            svc.state = ServiceState::Failed; 
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
            let current_time_after_wait = syscalls::get_system_time();
            let min_delay = services.iter()
                .filter(|s| s.service.next_start_time > current_time_after_wait)
                .map(|s| s.service.next_start_time.saturating_sub(current_time_after_wait))
                .min().unwrap_or(0);

            if min_delay > 0 {
                println!("[SUP] Geri çekilme bekleniyor... {}ms uyku.", min_delay);
                syscalls::task_sleep(min_delay); 
            } else {
                syscalls::task_yield();
            }
        } else {
            syscalls::task_yield();
        }
    }
    
    // Güvenli Kapatma
    task_exit(0); 
}
