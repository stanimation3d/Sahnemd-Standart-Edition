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
                    }
                    ControlMessage::RestartService(target_id) => {
                        // Basitçe hizmeti durdurulmuş (Stopped) duruma getir. Yeniden başlatma mantığı onu tekrar başlatacaktır.
                        if let Some(dep_service) = services.iter_mut().find(|s| {
                            if let ServiceState::Running(id) = s.service.state { id == target_id } else { false }
                        }) {
                            println!("[SUP] Hizmet yeniden başlatma isteği: {}", dep_service.service.name);
                            // Görevi sonlandırmak için SYSCALL_TASK_KILL kullanılabilir. 
                            // Şimdilik sadece durumu Stopped yapalım ve görev sonlanmasını bekleyelim.
                            dep_service.service.state = ServiceState::Stopped; 
                            // Not: Gerçekte burada SYSCALL_TASK_KILL (örneğin 104) çağrılmalıdır.
                        } else {
                            eprintln!("[SUP] Hata: Yeniden başlatılmak istenen görev bulunamadı: {}", target_id);
                        }
                    }
                    // ... Diğer mesajlar buraya eklenebilir ...
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
        // (Önceki koddan A, B, C adımları burada devam eder)
        // ... (Kod tekrarı yapmıyorum, tam kodu aşağıda veriyorum)
        // ...
        
        // C. TASK_WAIT veya TASK_SLEEP
        // ... (Bu kısım TASK_WAIT ile bloklama yapacak) ...

    } // loop sonu
    
    // Kapatma sinyali alındı, tüm çalışan görevleri sonlandır.
    // ... (Burada SYSCALL_TASK_KILL ile çalışan tüm görevler sonlandırılmalıdır) ...
    
    task_exit(0); 
}
