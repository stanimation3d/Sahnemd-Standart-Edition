// src/main.rs (GÜNCELLENMİŞ)

// ... Modül tanımları ve importlar (Değişmedi) ...

// --- Dışarıdan Alınanlar ---
use core::panic::PanicInfo;
use crate::service::{Service, RestartPolicy};
use crate::supervisor::run_supervisor_loop;
use crate::deps::{Dependency, DependencyService};
use crate::syscalls::task_exit; 
use crate::syscalls; 
use crate::ipc::{self, ChannelHandle}; // YENİ: IPC import edildi

// --- Global Veri: Init Sistemi Hizmetleri ---

// ... SERVICES tanımı (Değişmedi) ...

// YENİ: Yönetim Kanalı Handle'ı. Init sisteminin dışarıdan komut almasını sağlar.
static mut CONTROL_CHANNEL: Option<ChannelHandle> = None;

// Ana Giriş Noktası
#[no_mangle]
pub extern "C" fn main() -> ! {
    let init_task_id = syscalls::get_task_id();
    
    println!("\n=====================================");
    println!("SAHNEMD INIT SİSTEMİ BAŞLATILIYOR (ID: {})", init_task_id);

    // YENİ: Yönetim kanalını oluştur
    match ipc::create_control_channel() {
        Ok(handle) => {
            unsafe { CONTROL_CHANNEL = Some(handle); }
            println!("Kontrol Kanalı Başarılı. Handle: {}", handle.0);
        },
        Err(e) => {
            eprintln!("HATA: Kontrol Kanalı Oluşturulamadı: {}", e);
            task_exit(1); // Kritik hata, init sistemini sonlandır
        }
    }
    println!("=====================================\n");


    unsafe {
        // run_supervisor_loop'a kanal handle'ını geçirmemiz gerekiyor.
        // Veya global statik üzerinden erişmesini sağlayabiliriz. Global static kullanalım:
        run_supervisor_loop(SERVICES.as_mut(), CONTROL_CHANNEL.unwrap());
    }

    println!("Süpervizör döngüsü sonlandı. Sistemi kapatma başlatılıyor.");
    task_exit(0); 
}

// ... Panic İşleyici (Değişmedi) ...
