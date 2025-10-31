// src/main.rs

#![no_std]
#![no_main]

// --- Modülleri Tanımla ---
// Tüm modüllerin birbiriyle konuşabilmesi için crate kökünde tanımlanması gerekir.
mod syscalls;
mod service;
mod supervisor;
mod deps;
// mod resource; // Şimdilik sadece init mantığı için gerekli olanları ekliyoruz.
// mod ipc;

// --- Dışarıdan Alınanlar ---
use core::panic::PanicInfo;
use crate::service::Service;
use crate::supervisor::run_supervisor_loop;
use crate::deps::{Dependency, DependencyService};
use crate::syscalls::task_exit; // Çıkış için

// --- Global Veri: Init Sistemi Hizmetleri ---

// Bu dizideki hizmetler, Sahnemd başlatıldığında yönetilecektir.
// Bu örnekte, 'filesys' önce başlar, sonra 'shell' onu bekler.
static mut SERVICES: [DependencyService; 2] = [
    DependencyService {
        service: Service::new(
            "filesys", 
            "/bin/fs_driver", // Örnek dosya sistemi sürücüsü
            &[] // Argüman yok
        ),
        dependencies: &[], // İlk hizmetin bağımlılığı yok
    },
    DependencyService {
        service: Service::new(
            "sahne_shell", 
            "/bin/sh", 
            &["-i"] // İnteraktif mod
        ),
        // Kabuk başlamadan önce dosya sisteminin ('filesys') çalışıyor olması gerekir.
        dependencies: &[Dependency::Requires("filesys")], 
    },
];

// --- Ana Giriş Noktası ---

#[no_mangle]
pub extern "C" fn main() -> ! {
    // Sahnemd'in ilk görev kimliğini alalım. (Genellikle init sisteminin ID'si 1'dir)
    let init_task_id = syscalls::get_task_id();
    
    // Basit bir çıktı ile başladığımızı belirtelim.
    // (Varsayım: Sahne64 çekirdeği temel bir konsol çıktısı mekanizmasına sahip,
    // ya da bu bir syscall ile yapılacaktır. Şimdilik print! varsayalım.)
    println!("\n=====================================");
    println!("SAHNEMD INIT SİSTEMİ BAŞLATILIYOR (ID: {})", init_task_id);
    println!("=====================================\n");

    // Tüm hizmetlerimizi run_supervisor_loop'a geçiriyoruz.
    // Güvenli olmayan (unsafe) bir blok içinde geçiriyoruz, çünkü global statik veriye erişiyoruz.
    unsafe {
        // Init döngüsünü başlat: Bu fonksiyon asla geri dönmez (veya sistemi kapatır).
        run_supervisor_loop(SERVICES.as_mut());
    }

    // Eğer run_supervisor_loop geri dönerse (yani sistemin kapatılması gerekiyorsa):
    println!("Süpervizör döngüsü sonlandı. Sistemi kapatma başlatılıyor.");
    
    // Sistem kapatma durum kodu (Örn: 0 = Başarılı Kapatma)
    task_exit(0); 
}

// --- Panic İşleyici ---

// no-std ortamında, program çöktüğünde (panic) ne olacağını tanımlamamız gerekir.
// Init sistemi çökerse, bu kritik bir hatadır.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        eprintln!(
            "!!! SAHNEMD KRİTİK HATA !!!\nKonum: {}:{}\nMesaj: {:?}", 
            location.file(), 
            location.line(), 
            info.message().unwrap_or(&format_args!("Bilinmeyen Hata"))
        );
    } else {
        eprintln!("!!! SAHNEMD KRİTİK HATA !!! Bilinmeyen Konum.");
    }
    
    // Panik durumunda init sistemi daha fazla çalışamaz. Hata koduyla sonlandırıyoruz.
    // (Örn: 1 = Genel Hata)
    task_exit(1);
}