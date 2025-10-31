// src/syscalls.rs (GÜNCELLENMİŞ)

use core::fmt;

// ... TaskId tanımı ve Syscall Sabitleri (Değişmedi) ...

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskId(pub u64);

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Task({})", self.0)
    }
}

// --- Sahne64 Sistem Çağrısı Sabitleri ---
pub mod sys_const {
    pub const SYSCALL_TASK_SPAWN: u64 = 3;
    pub const SYSCALL_TASK_EXIT: u64 = 4;
    pub const SYSCALL_GET_TASK_ID: u64 = 9;
    pub const SYSCALL_TASK_SLEEP: u64 = 10; // YENİ EKLENDİ
    pub const SYSCALL_GET_SYSTEM_TIME: u64 = 16; // YENİ EKLENDİ
    pub const SYSCALL_TASK_YIELD: u64 = 101;
    pub const SYSCALL_TASK_WAIT: u64 = 105;
}

// --- Düşük Seviyeli Syscall Fonksiyonu (Değişmedi) ---
#[link(name = "sahne64_kernel", kind = "static")]
extern "C" {
    pub fn syscall6(
        sys_id: u64,
        arg1: u64,
        arg2: u64,
        arg3: u64,
        arg4: u64,
        arg5: u64,
        arg6: u64,
    ) -> u64;
}

// --- Yüksek Seviyeli Sarmalayıcılar ---

// ... get_task_id, task_exit, task_yield, task_wait (Değişmedi) ...

// Görevi belirli bir süre uyutur (milisaniye cinsinden varsayalım).
/// SYSCALL_TASK_SLEEP: u64 = 10;
/// Arg1: Uyutma süresi (u64, milisaniye cinsinden)
pub fn task_sleep(ms: u64) {
    unsafe {
        syscall6(sys_const::SYSCALL_TASK_SLEEP, ms, 0, 0, 0, 0, 0);
    }
}

/// Sistem saatini milisaniye cinsinden alır (Varsayım).
/// SYSCALL_GET_SYSTEM_TIME: u64 = 16;
pub fn get_system_time() -> u64 {
    unsafe {
        syscall6(sys_const::SYSCALL_GET_SYSTEM_TIME, 0, 0, 0, 0, 0, 0)
    }
}

// task_wait fonksiyonu (Değişmedi)
pub fn task_wait(task_id: Option<TaskId>, status_ptr: *mut u64) -> Result<TaskId, u64> {
    let target_id = task_id.map(|id| id.0).unwrap_or(0);
    
    let result = unsafe {
        syscall6(
            sys_const::SYSCALL_TASK_WAIT,
            target_id,         
            status_ptr as u64, 
            0, 0, 0, 0
        )
    };

    if result > 0 {
        Ok(TaskId(result))
    } else {
        Err(result)
    }
}
