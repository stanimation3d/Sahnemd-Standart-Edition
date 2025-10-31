// src/deps.rs

use crate::service::{Service, ServiceState};
// supervisor::spawn_task'e bağımlılık kalktı.

/// Bir hizmetin başlatılması için gereken bağımlılıkları tanımlar.
pub enum Dependency {
    Requires(&'static str), 
    Conflicts(&'static str),
    ResourceReady(&'static str),
}

/// Bağımlılıkları olan bir hizmet tanımı.
#[derive(Debug)]
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
                if let Some(required_svc) = all_services.iter().find(|s| s.service.name == *required_name) {
                    if !matches!(required_svc.service.state, ServiceState::Running(_)) {
                        return false;
                    }
                } else {
                    eprintln!("Hata: {} bilinmeyen hizmete bağımlı!", dep_service.service.name);
                    return false;
                }
            }
            _ => continue,
        }
    }
    true 
}
