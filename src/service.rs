// src/service.rs

/// Sahne64'te bir görevi (Task) temsil eden kimlik.
/// Başlatılan her program bu Task ID'ye sahip olacaktır.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskId(pub u64);

/// Bir hizmetin olası durumları.
#[derive(Debug, PartialEq)]
pub enum ServiceState {
    Stopped,      // Henüz başlatılmadı veya durduruldu
    Starting,     // Başlatılıyor (Bağımlılıklar bekleniyor olabilir)
    Running(TaskId), // Başarılı bir şekilde çalışıyor (Task ID'si ile birlikte)
    Failed,       // Başlatılamadı veya beklenmedik şekilde sonlandı
    Waiting,      // Başka bir hizmetin sonlanmasını veya bir olayı bekliyor
}

/// Init sistemi tarafından yönetilecek bir hizmetin temel tanımı.
#[derive(Debug)]
pub struct Service {
    pub name: &'static str,             // Hizmetin tanımlayıcı adı (Örn: "sahne_shell")
    pub path: &'static str,             // Çalıştırılacak ikili dosyanın yolu
    pub args: &'static [&'static str],  // Başlatma argümanları
    pub state: ServiceState,            // Mevcut çalışma durumu
    // Diğer alanlar: restart_policy, dependencies, vb.
}

impl Service {
    /// Yeni bir hizmet tanımı oluşturur.
    pub const fn new(
        name: &'static str, 
        path: &'static str, 
        args: &'static [&'static str]
    ) -> Self {
        Service {
            name,
            path,
            args,
            state: ServiceState::Stopped,
        }
    }
}