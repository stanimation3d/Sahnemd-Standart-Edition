// src/deps.rs

use crate::service::{Service, ServiceState};
use crate::supervisor::spawn_task;

/// Bir hizmetin başlatılması için gereken bağımlılıkları tanımlar.
pub enum Dependency {
    // Başka bir hizmetin çalışır durumda olmasını gerektirir
    Requires(&'static str), 
    // Başka bir hizmetin durmasını gerektirir (Nadiren kullanılır)
    Conflicts(&'static str),
    // Belirli bir kaynağın (Örn: Ağ veya Dosya Sistemi) hazır olmasını gerektirir
    ResourceReady(&'static str),
}

/// Bağımlılıkları olan bir hizmet tanımı.
pub struct DependencyService {
    pub service: Service,
    pub dependencies: &'static [Dependency],
}

/// Tüm hizmetler listesinde bir hizmetin bağımlılıklarının karşılanıp karşılanmadığını kontrol eder.
pub fn check_dependencies(
    dep_service: &DependencyService, 
    all_services: &[DependencyService]
) -> bool {
    for dep in dep_service.dependencies.iter() {
        match dep {
            Dependency::Requires(required_name) => {
                // Gerekli hizmeti listede bul
                if let Some(required_svc) = all_services.iter().find(|s| s.service.name == *required_name) {
                    // Eğer gereken hizmet Running durumunda değilse, bağımlılık karşılanmamış demektir.
                    if !matches!(required_svc.service.state, ServiceState::Running(_)) {
                        println!("Bağımlılık bekleniyor: {} -> {}", dep_service.service.name, required_name);
                        return false;
                    }
                } else {
                    // Bağımlı olunan hizmet listede yoksa bu bir konfigürasyon hatasıdır.
                    eprintln!("Hata: {} bilinmeyen hizmete bağımlı!", dep_service.service.name);
                    return false;
                }
            }
            // Diğer bağımlılık türleri (Conflicts, ResourceReady) burada kontrol edilir.
            _ => continue,
        }
    }
    true // Tüm bağımlılıklar karşılandı
}

/// Hizmetlerinizi bağımlılık sırasına göre başlatmak için kullanılır.
pub fn try_start_service(dep_service: &mut DependencyService, all_services: &mut [DependencyService]) {
    if dep_service.service.state == ServiceState::Stopped {
        // 1. Bağımlılıkları kontrol et
        if check_dependencies(dep_service, all_services) {
            
            // 2. Bağımlılıklar karşılandıysa görevi başlat
            dep_service.service.state = ServiceState::Starting;
            match spawn_task(&dep_service.service) {
                Ok(task_id) => {
                    println!(" -> Başarılı. Task ID: {:?}", task_id);
                    dep_service.service.state = ServiceState::Running(task_id);
                },
                Err(e) => {
                    eprintln!(" -> Başarısız! Hata Kodu: {}", e);
                    dep_service.service.state = ServiceState::Failed;
                }
            }
        } else {
            // Bağımlılıklar karşılanmadı, bir sonraki döngüyü bekle
            dep_service.service.state = ServiceState::Waiting;
        }
    }
}