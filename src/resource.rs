// src/resource.rs

use crate::syscalls::{self, sys_const};
use core::fmt;

/// Sahne64 tarafından döndürülen bir kaynağa erişim tanıtıcısıdır (File Descriptor benzeri).
/// Bir struct içinde tutulması, kullanımını daha güvenli hale getirir.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceHandle(pub u64);

/// Kaynak işlemlerinden dönebilecek standart hata türleri.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResourceError {
    InvalidHandle,      // Geçersiz veya kapanmış tanıtıcı
    NotFound,           // Kaynak bulunamadı (ACQUIRE için)
    PermissionDenied,   // Erişim izni yok
    IoError(u64),       // Diğer I/O hataları
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceError::InvalidHandle => write!(f, "Geçersiz Kaynak Tanıtıcısı"),
            ResourceError::NotFound => write!(f, "Kaynak Bulunamadı"),
            ResourceError::PermissionDenied => write!(f, "Erişim Reddedildi"),
            ResourceError::IoError(code) => write!(f, "I/O Hatası ({})", code),
        }
    }
}

/// Kaynak Yönetimi için Yüksek Seviyeli Fonksiyonlar
pub fn acquire_resource(path: &str, flags: u64) -> Result<ResourceHandle, ResourceError> {
    let path_bytes = path.as_bytes();
    let path_ptr = path_bytes.as_ptr() as u64;
    let path_len = path_bytes.len() as u64;

    let result = unsafe {
        // SYSCALL_RESOURCE_ACQUIRE = 5
        // Arg1: Yol İşaretçisi, Arg2: Yol Uzunluğu, Arg3: Bayraklar (Flags)
        syscalls::syscall6(
            sys_const::SYSCALL_RESOURCE_ACQUIRE,
            path_ptr,
            path_len,
            flags,
            0, 0, 0
        )
    };

    if result > 0 {
        Ok(ResourceHandle(result))
    } else {
        // Hata kodlarını ResourceError'a çevirme mantığı burada olmalı. 
        // Basitlik için sadece 0 (başarısız) varsayalım ve hata kodunu taşıyalım.
        match result {
            // ... Sahne64'e özgü hata kodları buraya gelecektir ...
            // Şimdilik sadece genel bir I/O hatası döndürelim:
            _ => Err(ResourceError::IoError(result)), 
        }
    }
}

pub fn release_resource(handle: ResourceHandle) {
    unsafe {
        // SYSCALL_RESOURCE_RELEASE = 8
        syscalls::syscall6(
            sys_const::SYSCALL_RESOURCE_RELEASE,
            handle.0, // Arg1: Handle
            0, 0, 0, 0, 0
        );
    }
}

// Okuma ve Yazma fonksiyonları da benzer şekilde yazılmalıdır:
// pub fn resource_read(handle: ResourceHandle, buffer: &mut [u8]) -> Result<usize, ResourceError> { ... }
// pub fn resource_write(handle: ResourceHandle, data: &[u8]) -> Result<usize, ResourceError> { ... }