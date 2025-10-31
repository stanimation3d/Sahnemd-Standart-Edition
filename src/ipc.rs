// src/ipc.rs (GÜNCELLENMİŞ)

use crate::syscalls::{self, sys_const, TaskId};
use core::fmt;

/// IPC kanalı için kullanılan tanıtıcı.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelHandle(pub u64);

/// Init sisteminin almayı beklediği yönetim komutları.
#[derive(Debug)]
pub enum ControlMessage {
    Shutdown,                       // Sistemi kapat (Graceful exit)
    RestartService(TaskId),         // Belirli bir görevi yeniden başlat
    RequestStatus,                  // Tüm hizmetlerin durumunu iste (Cevap IPC ile gönderilmeli)
    StopService(TaskId),            // Belirli bir görevi durdur
    Unknown,                        // Tanımlanamayan mesaj
}

/// Yönetim kanalı oluşturur.
pub fn create_control_channel() -> Result<ChannelHandle, u64> {
    let result = unsafe {
        // SYSCALL_CHANNEL_CREATE = 106
        syscalls::syscall6(
            sys_const::SYSCALL_CHANNEL_CREATE,
            0, 0, 0, 0, 0, 0
        )
    };

    if result > 0 {
        Ok(ChannelHandle(result))
    } else {
        Err(result) 
    }
}

/// Yönetim kanalından gelen mesajı alır ve ControlMessage'a çevirir.
/// NOT: Bu fonksiyonun bloklayıcı olmaması önemlidir, böylece görev bekleme (TASK_WAIT) 
/// ile çakışmaz.
/// Varsayım: Çekirdek, `SYSCALL_TASK_WAIT` veya `SYSCALL_CHANNEL_RECEIVE`'dan birinde 
/// sinyal beklerken diğerini kesintiye uğratacak bir `poll` mekanizması sunmalıdır. 
/// Şimdilik `channel_receive`'ı bloklamayan (non-blocking) bir çağrı olarak varsayalım.
pub fn channel_receive_non_blocking(handle: ChannelHandle) -> Result<ControlMessage, u64> {
    
    // Varsayım: Mesaj yapısı: [U64 ID | U64 Arg1]
    let mut buffer = [0u8; 16]; 
    let buffer_ptr = buffer.as_mut_ptr() as u64;
    let buffer_len = buffer.len() as u64;

    let result = unsafe {
        // SYSCALL_CHANNEL_RECEIVE = 109
        // Arg1: Kanal Handle'ı, Arg2: Tampon İşaretçisi, Arg3: Tampon Uzunluğu
        // Arg4: Bayraklar (Örn: Bloklama/Bloklamama bayrağı = 100 varsayalım, gerçek API'ye göre değişir)
        syscalls::syscall6(
            sys_const::SYSCALL_CHANNEL_RECEIVE,
            handle.0,
            buffer_ptr,
            buffer_len,
            100, // Non-blocking flag varsayımı
            0, 0
        )
    };

    if result == 0 {
        // Bloklamayan çağrı başarılı oldu ama veri yok.
        return Err(0); // Veya API'nin "veri yok" hata kodunu döndür.
    } else if result as i64 < 0 {
        // Hata
        return Err(result);
    } 
    
    // Basit deserialization (Seri Çözme): 
    // buffer'ın ilk 8 byte'ı komut ID, sonraki 8 byte'ı ise argümandır.
    let command_id = u64::from_le_bytes(buffer[0..8].try_into().unwrap_or([0; 8]));
    let arg1 = u64::from_le_bytes(buffer[8..16].try_into().unwrap_or([0; 8]));

    match command_id {
        1 => Ok(ControlMessage::Shutdown),
        2 => Ok(ControlMessage::RestartService(TaskId(arg1))),
        3 => Ok(ControlMessage::RequestStatus),
        4 => Ok(ControlMessage::StopService(TaskId(arg1))),
        _ => Ok(ControlMessage::Unknown),
    }
}
