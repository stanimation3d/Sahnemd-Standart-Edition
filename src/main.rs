// src/main.rs

#![no_std]
#![no_main]

// --- Modülleri Tanımla ---
mod syscalls;
mod service;
mod supervisor;
mod deps;
// mod resource; 
// mod ipc;

// --- Dışarıdan Alınanlar ---
use core::panic::PanicInfo;
use crate::service::{Service, RestartPolicy};
use crate::supervisor::run_supervisor_loop;
use crate::deps::{Dependency, DependencyService};
use crate::syscalls::task_exit; 
use crate::syscalls; 

// --- Global Veri: Init Sistemi Hizmetleri ---

static mut SERVICES: [DependencyService; 2] = [
    DependencyService {
        service: Service::new(
            "filesys", 
            "/bin/fs_driver", 
            &[], 
            RestartPolicy::OnFailure, // Hata durumunda yeniden başlat
        ),
        dependencies: &[], 
    },
    DependencyService {
        service: Service::new(
            "sahne_shell", 
            "/bin/sh", 
            &["-i"], 
            RestartPolicy::Always, // Her zaman yeniden başlat
        ),
        dependencies: &[Dependency::Requires("filesys")], 
    },
];

// --- Ana Giriş Noktası ---

#[no_mangle]
pub extern "C" fn main() -> ! {
    let init_task_id = syscalls::get_task_id();
    
    println!("\n=====================================");
    println!("SAHNEMD INIT SİSTEMİ BAŞLATILIYOR (ID: {})", init_task_id);
    println!("=====================================\n");

    unsafe {
        run_supervisor_loop(SERVICES.as_mut());
    }

    println!("Süpervizör döngüsü sonlandı. Sistemi kapatma başlatılıyor.");
    task_exit(0); 
}

// --- Panic İşleyici ---

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
    
    task_exit(1);
}
