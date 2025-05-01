// src/utils.rs

use std::io::{self, Read, Seek, SeekFrom};
use std::mem::size_of;
use byteorder::{LittleEndian, ReadBytesExt};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VisioUtilsError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid UTF-16 string")]
    InvalidUtf16,
    #[error("Invalid data format")]
    InvalidFormat,
}

/// Читает строку в кодировке UTF-16LE из потока
pub fn read_utf16_string<R: Read>(reader: &mut R, len: usize) -> Result<String, VisioUtilsError> {
    let mut buf = vec![0u8; len * 2];
    reader.read_exact(&mut buf)?;
    
    String::from_utf16(
        &buf.chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect::<Vec<_>>()
    ).map_err(|_| VisioUtilsError::InvalidUtf16)
}

/// Читает 16-битное целое с проверкой
pub fn read_u16_checked<R: Read>(reader: &mut R) -> Result<u16, VisioUtilsError> {
    reader.read_u16::<LittleEndian>().map_err(Into::into)
}

/// Читает 32-битное целое с проверкой
pub fn read_u32_checked<R: Read>(reader: &mut R) -> Result<u32, VisioUtilsError> {
    reader.read_u32::<LittleEndian>().map_err(Into::into)
}

/// Читает 64-битное целое с проверкой
pub fn read_u64_checked<R: Read>(reader: &mut R) -> Result<u64, VisioUtilsError> {
    reader.read_u64::<LittleEndian>().map_err(Into::into)
}

/// Пропускает указанное количество байт в потоке
pub fn skip_bytes<R: Read + Seek>(reader: &mut R, count: u64) -> Result<(), VisioUtilsError> {
    reader.seek(SeekFrom::Current(count as i64))?;
    Ok(())
}

/// Читает GUID из потока
pub fn read_guid<R: Read>(reader: &mut R) -> Result<[u8; 16], VisioUtilsError> {
    let mut guid = [0u8; 16];
    reader.read_exact(&mut guid)?;
    Ok(guid)
}

/// Конвертирует OLE-дату в SystemTime
pub fn ole_date_to_system_time(ole_date: f64) -> Option<std::time::SystemTime> {
    // Реализация конвертации OLE-даты (как в оригинале)
    unimplemented!()
}

/// Проверяет сигнатуру файла
pub fn check_file_signature<R: Read + Seek>(
    reader: &mut R, 
    signature: &[u8]
) -> Result<bool, VisioUtilsError> {
    let mut buf = vec![0u8; signature.len()];
    reader.read_exact(&mut buf)?;
    reader.seek(SeekFrom::Start(0))?;
    Ok(buf == signature)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_utf16_string() {
        let data = b"H\0e\0l\0l\0o\0"; // "Hello" в UTF-16LE
        let mut cursor = Cursor::new(data);
        let s = read_utf16_string(&mut cursor, 5).unwrap();
        assert_eq!(s, "Hello");
    }

    #[test]
    fn test_read_u32_checked() {
        let data = [0x78, 0x56, 0x34, 0x12]; // 0x12345678
        let mut cursor = Cursor::new(data);
        let val = read_u32_checked(&mut cursor).unwrap();
        assert_eq!(val, 0x12345678);
    }
}