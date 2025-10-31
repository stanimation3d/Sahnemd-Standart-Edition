// src/service.rs (GÜNCELLENMİŞ)

use crate::syscalls::TaskId;

/// Bir hizmetin hangi koşullarda yeniden başlatılacağını belirler.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RestartPolicy {
    Always,      // Her sonlanmada yeniden başlat (başarılı/başarısız fark etmez)
    OnFailure,   // Sadece hata koduyla (exit status != 0) sonlanırsa yeniden başlat
    Never,       // Asla yeniden başlatma (Varsayılan init sistemi davranışı)
}

/// Bir hizmetin olası durumları.
#[derive(Debug, PartialEq)]
pub enum ServiceState {
    Stopped,      
    Starting,     
    Running(TaskId), 
    Failed,       
    Waiting,      
}

/// Init sistemi tarafından yönetilecek bir hizmetin temel tanımı.
#[derive(Debug)]
pub struct Service {
    pub name: &'static str,             
    pub path: &'static str,             
    pub args: &'static [&'static str],  
    pub state: ServiceState,            
    pub restart_policy: RestartPolicy, // YENİ: Yeniden başlatma politikası
}

impl Service {
    pub const fn new(
        name: &'static str, 
        path: &'static str, 
        args: &'static [&'static str],
        restart_policy: RestartPolicy, // YENİ: Parametre eklendi
    ) -> Self {
        Service {
            name,
            path,
            args,
            state: ServiceState::Stopped,
            restart_policy, // Yeni alan atandı
        }
    }
}
