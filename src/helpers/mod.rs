use cfb::{self, Stream};
use std::io::{self, Read, Seek, SeekFrom};
use std::ops::Range;

/// Тип для представления потока ввода, аналогичный librevenge::RVNGInputStream
pub trait RVNGInputStream: Read + Seek {}
impl<T: Read + Seek> RVNGInputStream for T {}

/// Внутренний поток для работы с VSD данными
pub struct VSDInternalStream {
    buffer: Vec<u8>,
    offset: usize,
}

impl VSDInternalStream {
    /// Создает новый VSDInternalStream из входного потока
    pub fn new(input: &mut dyn RVNGInputStream, size: usize, compressed: bool) -> io::Result<Self> {
        let mut buffer = vec![0; size];
        let bytes_read = input.read(&mut buffer)?;

        if bytes_read < 2 {
            return Ok(Self {
                buffer: Vec::new(),
                offset: 0,
            });
        }

        if !compressed {
            buffer.truncate(bytes_read);
            Ok(Self { buffer, offset: 0 })
        } else {
            Self::decompress_buffer(&buffer[..bytes_read])
        }
    }

    /// Декомпрессия буфера по алгоритму VSD
    fn decompress_buffer(input: &[u8]) -> io::Result<Self> {
        let mut output = Vec::new();
        let mut history = [0u8; 4096];
        let mut pos = 0;
        let mut offset = 0;

        while offset < input.len() {
            let flag = input[offset];
            offset += 1;

            if offset >= input.len() {
                break;
            }

            let mut mask = 1;
            for _ in 0..8 {
                if offset >= input.len() {
                    break;
                }

                if flag & mask != 0 {
                    // Просто копируем байт
                    history[pos % 4096] = input[offset];
                    output.push(input[offset]);
                    offset += 1;
                    pos += 1;
                } else {
                    // Копируем из истории
                    if offset + 1 >= input.len() {
                        break;
                    }

                    let addr1 = input[offset] as usize;
                    let addr2 = input[offset + 1] as usize;
                    offset += 2;

                    let length = (addr2 & 0x0F) + 3;
                    let mut pointer = ((addr2 & 0xF0) << 4) | addr1;

                    if pointer > 4078 {
                        pointer -= 4078;
                    } else {
                        pointer += 18;
                    }

                    for j in 0..length {
                        let src_pos = (pointer + j) % 4096;
                        let value = history[src_pos];
                        history[(pos + j) % 4096] = value;
                        output.push(value);
                    }
                    pos += length;
                }

                mask <<= 1;
            }
        }

        Ok(Self {
            buffer: output,
            offset: 0,
        })
    }

    /// Проверяет, достигнут ли конец потока
    pub fn is_end(&self) -> bool {
        self.offset >= self.buffer.len()
    }

    /// Возвращает текущую позицию
    pub fn tell(&self) -> u64 {
        self.offset as u64
    }
}

impl Read for VSDInternalStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let bytes_available = self.buffer.len() - self.offset;
        let bytes_to_read = buf.len().min(bytes_available);

        if bytes_to_read == 0 {
            return Ok(0);
        }

        buf[..bytes_to_read]
            .copy_from_slice(&self.buffer[self.offset..self.offset + bytes_to_read]);
        self.offset += bytes_to_read;

        Ok(bytes_to_read)
    }
}

impl Seek for VSDInternalStream {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_offset = match pos {
            SeekFrom::Start(offset) => offset as i64,
            SeekFrom::End(offset) => (self.buffer.len() as i64) + offset,
            SeekFrom::Current(offset) => (self.offset as i64) + offset,
        };

        self.offset = if new_offset < 0 {
            0
        } else if new_offset > (self.buffer.len() as i64) {
            self.buffer.len()
        } else {
            new_offset as usize
        };

        Ok(self.offset as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    struct MockInputStream {
        data: Vec<u8>,
        pos: usize,
    }

    impl Read for MockInputStream {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let bytes_to_copy = buf.len().min(self.data.len() - self.pos);
            buf[..bytes_to_copy].copy_from_slice(&self.data[self.pos..self.pos + bytes_to_copy]);
            self.pos += bytes_to_copy;
            Ok(bytes_to_copy)
        }
    }

    impl Seek for MockInputStream {
        fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
            let new_pos = match pos {
                SeekFrom::Start(offset) => offset as i64,
                SeekFrom::End(offset) => (self.data.len() as i64) + offset,
                SeekFrom::Current(offset) => (self.pos as i64) + offset,
            };

            self.pos = if new_pos < 0 {
                0
            } else if new_pos > (self.data.len() as i64) {
                self.data.len()
            } else {
                new_pos as usize
            };

            Ok(self.pos as u64)
        }
    }

    #[test]
    fn test_stream_operations() {
        let data = vec![1, 2, 3, 4, 5];
        let mut input = Cursor::new(data.clone());
        let mut stream = VSDInternalStream::new(&mut input, data.len(), false).unwrap();

        // Test read
        let mut buf = [0; 3];
        assert_eq!(stream.read(&mut buf).unwrap(), 3);
        assert_eq!(buf, [1, 2, 3]);

        // Test seek
        assert_eq!(stream.seek(SeekFrom::Start(1)).unwrap(), 1);
        assert_eq!(stream.read(&mut buf).unwrap(), 3);
        assert_eq!(buf, [2, 3, 4]);

        // Test is_end
        assert!(!stream.is_end());
        stream.seek(SeekFrom::End(0)).unwrap();
        assert!(stream.is_end());
    }
}
