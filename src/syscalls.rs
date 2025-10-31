// src/syscalls.rs (GÜNCELLENMİŞ)

// Önceki koddan TaskId yapısını import ediyoruz.
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskId(pub u64);

// Sadece basit bir println yerine formatlama desteği için
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
    pub const SYSCALL_TASK_YIELD: u64 = 101;
    pub const SYSCALL_TASK_WAIT: u64 = 105; // YENİ: Görev bekleme
    // ... Diğer sabitler ...
}

// --- Düşük Seviyeli Syscall Fonksiyonu ---
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

// --- Kullanışlı Yüksek Seviyeli Sarmalayıcılar ---

// Mevcut görevin kimliğini alır.
pub fn get_task_id() -> u64 {
    unsafe {
        syscall6(sys_const::SYSCALL_GET_TASK_ID, 0, 0, 0, 0, 0, 0)
    }
}

// Mevcut görevi sonlandırır.
pub fn task_exit(status: u64) -> ! {
    unsafe {
        syscall6(sys_const::SYSCALL_TASK_EXIT, status, 0, 0, 0, 0, 0);
    }
    loop {} 
}

// CPU'yu başka bir göreve devret.
pub fn task_yield() {
    unsafe {
        syscall6(sys_const::SYSCALL_TASK_YIELD, 0, 0, 0, 0, 0, 0);
    }
}

/// SYSCALL_TASK_WAIT: Bir görevin sonlanmasını bekler.
///
/// Varsayım:
/// Arg1: Beklenecek Task ID'si. Eğer 0 ise, herhangi bir çocuk görev beklenir.
/// Arg2: Çıkış durumunu yazmak için işaretçi (&mut u64)
/// Arg3: Opsiyonel bayraklar (Örn: Bloklamayan bekleme)
/// Geri dönüş: Sonlanan görev ID'si, hata durumunda 0 veya hata kodu.
pub fn task_wait(task_id: Option<TaskId>, status_ptr: *mut u64) -> Result<TaskId, u64> {
    let target_id = task_id.map(|id| id.0).unwrap_or(0); // None ise 0 (herhangi bir çocuk)
    
    let result = unsafe {
        syscall6(
            sys_const::SYSCALL_TASK_WAIT,
            target_id,         // Arg1: Task ID (veya 0)
            status_ptr as u64, // Arg2: Çıkış durumunun yazılacağı pointer
            0, 0, 0, 0
        )
    };

    if result > 0 {
        Ok(TaskId(result)) // Sonlanan Task ID'si
    } else {
        Err(result) // Hata kodu (Örn: çocuk yok, bekleme başarısız)
    }
}