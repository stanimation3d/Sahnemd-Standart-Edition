// src/ipc.rs

use crate::syscalls::{self, sys_const};

/// IPC kanalı için kullanılan tanıtıcı.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelHandle(pub u64);

/// IPC işlemlerinde kullanılacak mesaj türü (basitleştirilmiş).
pub enum IpcMessage {
    Shutdown,      // Sistemi kapat
    Restart(u64),  // Belirli bir hizmeti yeniden başlat (Task ID)
    StatusRequest, // Durum bilgisi isteği
    // ... Diğer mesaj türleri ...
}

/// Yeni bir mesaj kanalı oluşturur.
/// Geri dönüş değeri, bu kanalın Handle'ıdır.
pub fn create_channel() -> Result<ChannelHandle, u64> {
    let result = unsafe {
        // SYSCALL_CHANNEL_CREATE = 106. Parametre gerektirmeyebilir.
        syscalls::syscall6(
            sys_const::SYSCALL_CHANNEL_CREATE,
            0, 0, 0, 0, 0, 0
        )
    };

    if result > 0 {
        Ok(ChannelHandle(result))
    } else {
        Err(result) // Hata kodu
    }
}

/// Belirtilen kanala veri gönderir.
pub fn channel_send(handle: ChannelHandle, data: &[u8]) -> Result<(), u64> {
    let data_ptr = data.as_ptr() as u64;
    let data_len = data.len() as u64;

    let result = unsafe {
        // SYSCALL_CHANNEL_SEND = 108
        // Arg1: Kanal Handle'ı, Arg2: Veri İşaretçisi, Arg3: Veri Uzunluğu
        syscalls::syscall6(
            sys_const::SYSCALL_CHANNEL_SEND,
            handle.0,
            data_ptr,
            data_len,
            0, 0, 0
        )
    };

    if result == 0 {
        Ok(())
    } else {
        Err(result)
    }
}

/// Kanaldan mesaj alır (Bloklayabilir).
/// Buffer'a yazılan byte sayısını veya hata kodunu döndürür.
pub fn channel_receive(handle: ChannelHandle, buffer: &mut [u8]) -> Result<usize, u64> {
    let buffer_ptr = buffer.as_mut_ptr() as u64;
    let buffer_len = buffer.len() as u64;

    let result = unsafe {
        // SYSCALL_CHANNEL_RECEIVE = 109
        // Arg1: Kanal Handle'ı, Arg2: Tampon İşaretçisi, Arg3: Tampon Uzunluğu
        syscalls::syscall6(
            sys_const::SYSCALL_CHANNEL_RECEIVE,
            handle.0,
            buffer_ptr,
            buffer_len,
            0, 0, 0
        )
    };

    if result > 0 {
        Ok(result as usize) // Okunan byte sayısı
    } else {
        Err(result) // Hata kodu
    }
}