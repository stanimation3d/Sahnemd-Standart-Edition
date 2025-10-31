// src/service.rs

use crate::syscalls::TaskId;

/// Bir hizmetin hangi koşullarda yeniden başlatılacağını belirler.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RestartPolicy {
    Always,      
    OnFailure,   
    Never,       
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
    pub restart_policy: RestartPolicy, 
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
        }
    }
}
