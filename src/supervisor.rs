// src/supervisor.rs (GÜNCELLENMİŞ)

use core::slice;
use crate::syscalls::{self, sys_const, TaskId};
use crate::service::{Service, ServiceState};
use crate::deps::{DependencyService, check_dependencies}; // deps.rs'i kullanıyoruz!

/// # Süpervizörün Temel Fonksiyonları

/// Belirtilen Service tanımına göre yeni bir görevi (Task) Sahne64'te başlatır.
// (Önceki koddan bu fonksiyonu olduğu gibi koruyoruz)
pub fn spawn_task(service: &Service) -> Result<TaskId, u64> {
    let path_bytes = service.path.as_bytes();
    let path_ptr = path_bytes.as_ptr() as u64;
    let path_len = path_bytes.len() as u64;

    let result = unsafe {
        syscalls::syscall6(
            sys_const::SYSCALL_TASK_SPAWN,
            path_ptr,       // Arg1: Program yolu işaretçisi
            path_len,       // Arg2: Program yolu uzunluğu
            0, 0, 0, 0      
        )
    };

    if result > 0 {
        Ok(TaskId(result))
    } else {
        Err(result) 
    }
}


/// Tüm hizmetleri yöneten ana init döngüsünü başlatır ve görev sonlanmalarını izler.
pub fn run_supervisor_loop(services: &mut [DependencyService]) {
    
    println!("Sahnemd Init Sistemi Başlatılıyor...");
    let mut exit_status: u64 = 0;

    // 1. Hizmet Başlatma ve İzleme Döngüsü
    loop {
        let mut started_new = false;
        
        // A. Bağımlılıkları Kontrol Et ve Yeni Hizmetleri Başlat
        for dep_service in services.iter_mut() {
            if dep_service.service.state == ServiceState::Stopped || dep_service.service.state == ServiceState::Waiting {
                
                // deps.rs'ten try_start_service mantığını buraya taşıyalım 
                // ya da daha iyisi, loop içinde manuel kontrol yapalım:
                if check_dependencies(dep_service, services) {
                    println!("Hizmet başlatılıyor: {}", dep_service.service.name);
                    match spawn_task(&dep_service.service) {
                        Ok(task_id) => {
                            println!(" -> Başarılı. Task ID: {:?}", task_id);
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
        
        // B. Aktif Görev Kalmadıysa Sistemi Durdur
        let active_tasks = services.iter().any(|s| matches!(s.service.state, ServiceState::Running(_)));

        if !active_tasks {
            if started_new {
                // Yeni bir şey başlatıldı, bir sonraki döngüde bekle.
                continue;
            }
            // Başlatılacak veya çalışan görev kalmadıysa:
            println!("Tüm kullanıcı alanı görevleri sonlandı. Sahnemd kapatılıyor.");
            break;
        }

        // C. SYSCALL_TASK_WAIT ile Çocuk Görevlerin Sonlanmasını Bekle
        
        // `task_wait` fonksiyonuna çıkış durumunu yazmak için bir işaretçi veriyoruz.
        let wait_result = syscalls::task_wait(None, &mut exit_status as *mut u64);
        
        match wait_result {
            Ok(terminated_task_id) => {
                println!(
                    "GÖREV SONLANDI: Task ID: {} | Çıkış Durumu: {}", 
                    terminated_task_id, exit_status
                );
                
                // 1. Hangi hizmetin sonlandığını bul
                if let Some(dep_service) = services.iter_mut().find(|s| {
                    if let ServiceState::Running(id) = s.service.state {
                        id == terminated_task_id
                    } else {
                        false
                    }
                }) {
                    // 2. Hizmetin durumunu güncelle
                    println!(" -> Sonlanan hizmet: {}", dep_service.service.name);
                    
                    // Basit bir kural: Çıkış durumu 0 ise Durdu (Stopped), değilse Hata (Failed)
                    dep_service.service.state = if exit_status == 0 {
                        ServiceState::Stopped // Normal sonlanma
                    } else {
                        ServiceState::Failed  // Hatalı sonlanma
                    };
                    
                    // Burada yeniden başlatma politikaları (restart_policy) kontrol edilebilir.
                    
                } else {
                    eprintln!("UYARI: İzlenmeyen bir görev sonlandı: {}", terminated_task_id);
                }
            },
            Err(e) => {
                // Hata kodlarını kontrol et. EĞER HATA, BEKLENECEK GÖREV OLMADIĞI anlamına 
                // geliyorsa (ECHILD gibi), döngüye devam etmeli veya yield yapmalıyız.
                if e == 0 { // Genellikle 0 veya -1 gibi değerler beklenir. Sahne64 dokümantasyonuna bakılmalıdır.
                    // Şimdilik, hata varsa kısa bir süre bekleyelim ve CPU'yu devredelim.
                    syscalls::task_yield();
                } else {
                    eprintln!("SYSCALL_TASK_WAIT Hatası: {}", e);
                    // Çok ciddi bir hata, belki de init sistemi yeniden başlatılmalı.
                }
            }
        } // match wait_result sonu
    } // loop sonu
}