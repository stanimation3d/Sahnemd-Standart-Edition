// src/service.rs (GÜNCELLENMİŞ)

use crate::syscalls::TaskId;

// ... RestartPolicy ve ServiceState (Değişmedi) ...

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RestartPolicy {
    Always,      
    OnFailure,   
    Never,       
}

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
    pub restart_policy: RestartPolicy, 
    pub restart_count: u32,             // YENİ: Kaç kez yeniden başlatıldı
    pub next_start_time: u64,           // YENİ: Tekrar başlatılabileceği en erken zaman (milisaniye)
}

impl Service {
    pub const fn new(
        name: &'static str, 
        path: &'static str, 
        args: &'static [&'static str],
        restart_policy: RestartPolicy,
    ) -> Self {
        Service {
            name,
            path,
            args,
            state: ServiceState::Stopped,
            restart_policy,
            restart_count: 0,          // Sıfır ile başla
            next_start_time: 0,        // Hemen başlatılabilir
        }
    }

    /// Yeniden başlatma sayacını sıfırlar.
    pub fn reset_restart_count(&mut self) {
        self.restart_count = 0;
    }

    /// Başarısız bir başlatma/çalıştırma sonrası geri çekilme süresini ayarlar.
    pub fn set_backoff(&mut self, current_time: u64) {
        self.restart_count = self.restart_count.saturating_add(1);

        // Basit Üstel Geri Çekilme (Exponential Backoff): 
        // 1000ms, 2000ms, 4000ms, 8000ms... (Maksimum 32 saniye bekleme)
        let delay_ms = 1000 * 2u64.pow(self.restart_count.min(5) - 1); // min(5) = 16 saniyede dur
        
        // Yeniden başlatma bekleme süresini mevcut zamana ekleyelim.
        self.next_start_time = current_time.saturating_add(delay_ms);

        println!(
            "[SVC] Geri çekilme ayarlandı: {}ms gecikme. Yeni deneme zamanı: {}",
            delay_ms, self.next_start_time
        );
    }
}
