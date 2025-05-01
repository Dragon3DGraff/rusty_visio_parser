pub mod read_vsd {

    use crate::constants::SeekType;
    use crate::helpers::VSDInternalStream;
    use crate::vsd_constants::object_types::VSD_TRAILER_STREAM;
    use byteorder::{LittleEndian, ReadBytesExt};
    use cfb::{self, Stream};
    use std::io::Cursor;
    use std::io::{BufRead, Read, Seek, SeekFrom};
    use std::{fs::File, io::BufReader, path::Path};

    use crate::utils::{VisioUtilsError, read_u32_checked};
    use crate::vsd_constants::{cell_types, field_formats, object_types};

    #[derive(Debug, PartialEq)]
    pub struct Coordinate {
        pub x: i32,
        pub y: i32,
    }

    #[derive(Debug, Clone)]
    struct Pointer {
        pub type_name: u32,
        pub format: u32,
        pub offset: u32,
        pub length: u32,
        pub list_size: u16,
    }

    impl Pointer {
        fn new() -> Self {
            Self {
                type_name: 0,
                format: 0,
                offset: 0,
                length: 0,
                list_size: 0,
            }
        }
        fn read(&mut self, stream: &mut Cursor<Vec<u8>>) {
            self.type_name = stream.read_u32::<byteorder::LittleEndian>().unwrap();
            let _ = stream.read_u32::<byteorder::LittleEndian>().unwrap();
            self.format = stream.read_u32::<byteorder::LittleEndian>().unwrap();
            self.offset = stream.read_u32::<byteorder::LittleEndian>().unwrap();
            self.length = stream.read_u32::<byteorder::LittleEndian>().unwrap();
            self.list_size = stream.read_u16::<byteorder::LittleEndian>().unwrap();

            // self.format = reader.read_u16::<byteorder::LittleEndian>().unwrap() & 0x00FF;
            // let _ = reader.read_u32::<byteorder::LittleEndian>();
            // self.offset = reader.read_u32::<byteorder::LittleEndian>().unwrap();
            // self.length = reader.read_u32::<byteorder::LittleEndian>().unwrap();

            //               ptr.Type = readU32(input);
            //   input->seek(4, librevenge::RVNG_SEEK_CUR); // Skip dword
            //   ptr.Offset = readU32(input);
            //   ptr.Length = readU32(input);
            //   ptr.Format = readU16(input);
        }
    }

    // struct Stream {
    //     buffer: Vec<u8>,
    //     offset: usize,
    // }

    // impl Stream {
    //     pub fn read(&mut self, num_bytes: usize) -> Option<&[u8]> {
    //         if num_bytes == 0 {
    //             return None;
    //         }

    //         let num_bytes_to_read = if num_bytes < self.buffer.len() - self.offset {
    //             num_bytes
    //         } else {
    //             self.buffer.len() - self.offset
    //         };

    //         if num_bytes_to_read == 0 {
    //             return None;
    //         }

    //         let old_offset = self.offset;
    //         self.offset += num_bytes_to_read;

    //         Some(&self.buffer[old_offset..old_offset + num_bytes_to_read])
    //     }

    //     pub fn seek(&mut self, offset: i64, seek_type: SeekType) -> std::io::Result<()> {
    //         self.offset = match seek_type {
    //             SeekType::RVNG_SEEK_CUR => (self.offset as i64).checked_add(offset),
    //             SeekType::RVNG_SEEK_SET => Some(offset),
    //             SeekType::RVNG_SEEK_END => (self.buffer.len() as i64).checked_add(offset),
    //         }
    //         .ok_or_else(|| {
    //             std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid seek position")
    //         })?
    //         .try_into()
    //         .unwrap();

    //         // Проверка границ
    //         self.offset = if self.offset < 0 {
    //             0
    //         } else if self.offset > (self.buffer.len() as i64).try_into().unwrap() {
    //             self.buffer.len() as i64
    //         } else {
    //             self.offset.try_into().unwrap()
    //         } as usize;

    //         Ok(())
    //     }

    //     // fn seek(&mut self, offset: i128, seek_type: SeekType) -> i32 {
    //     //     let length = i128::try_from(self.buffer.len()).unwrap();

    //     //     if seek_type == SeekType::RVNG_SEEK_CUR {
    //     //         self.m_offset = self.m_offset.to_owned() + offset;
    //     //     } else if seek_type == SeekType::RVNG_SEEK_SET {
    //     //         self.m_offset = offset;
    //     //     } else if seek_type == SeekType::RVNG_SEEK_END {
    //     //         self.m_offset = length + offset;
    //     //     }
    //     //     if self.m_offset < 0 {
    //     //         self.m_offset = 0;
    //     //         return 1;
    //     //     }
    //     //     if self.m_offset > length {
    //     //         self.m_offset = length;
    //     //         return 1;
    //     //     }

    //     //     return 0;
    //     // }
    // }

    pub fn read_file(file_path: &Path) {
        let mut cf = cfb::open_rw(file_path).unwrap();

        for entry in cf.read_root_storage() {
            println!("Found stream: {} {} bytes", entry.name(), entry.len());
        }
        match cf.open_stream("VisioDocument") {
            Err(err) => {
                println!("{}", err)
            }
            Ok(mut stream) => {
                let mut buf: Vec<u8> = Vec::new();
                let _ = stream.read_to_end(&mut buf);

                // Анализ бинарных данных
                let doc = match parse_visio_document(&buf) {
                    Ok(doc) => doc,
                    Err(err) => {
                        println!("Error on parse doc {}", err);
                        {}
                    }
                };

                doc
            }
        }

        // match File::open(file_path) {
        //     Ok(file) => {
        //         println!("Файл успешно открыт.");

        //         let mut reader: BufReader<File> = BufReader::new(file);
        //         let data_bytes = reader.fill_buf().unwrap();
        //         println!("data_bytes {:?}", data_bytes.len());
        //         let is_vsd = check_is_vsd(data_bytes);

        //         if !is_vsd {
        //             return;
        //         }
        //         let mut pointer = Pointer {
        //             type_name: 0,
        //             offset: 0,
        //             length: 0,
        //             format: 0,
        //             list_size: 0,
        //         };

        //         reader.seek(SeekFrom::Start(26)).unwrap();

        //         let version = reader.read_u8();

        //         println!("Версия продукта {:?}", version.unwrap());

        //         reader.seek(SeekFrom::Start(0x24)).unwrap();

        //         pointer.read(&mut reader);

        //         println!("{:?}", pointer);

        //         reader.seek(SeekFrom::Start(0x82)).unwrap();
        //         println!("stream_position {:?}", reader.stream_position().unwrap());

        //         let pointerCount = reader.read_i16::<byteorder::LittleEndian>().unwrap();
        //         println!("stream_position {:?}", reader.stream_position().unwrap());

        //         println!("pointerCount {}", pointerCount)

        //         // let type_name = reader.read_u16::<byteorder::LittleEndian>();

        //         // println!("type_name {:?}", type_name.unwrap() & 0x00ff);

        //         // println!("stream_position {:?}", reader.stream_position().unwrap());

        //         // let format = reader.read_u16::<byteorder::LittleEndian>();

        //         // println!("format {:?}", format.unwrap());

        //         // println!("stream_position {:?}", reader.stream_position().unwrap());

        //         // let pos = reader.stream_position().unwrap();

        //         // let _ = reader.seek(SeekFrom::Start(pos + 4));

        //         // let offset = reader.read_u32::<byteorder::LittleEndian>().unwrap();

        //         // println!("offset {:?}", offset);

        //         // println!("stream_position {:?}", reader.stream_position().unwrap());

        //         // let length = reader.read_u32::<byteorder::LittleEndian>();

        //         // println!("length {:?}", length.unwrap());

        //         // println!("stream_position {:?}", reader.stream_position().unwrap());

        //         // let _ = reader.seek(SeekFrom::Start(offset.into()));

        //         // println!("stream_position {:?}", reader.stream_position().unwrap());
        //     }
        //     Err(err) => {
        //         eprintln!(
        //             "Ошибка открытия файла {:?}: {}",
        //             file_path.file_name().unwrap().to_owned(),
        //             err
        //         );
        //     }
        // };
    }

    fn parse_visio_document(data: &Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        // println!("parse_visio_document {} bytes", data.len());

        // println!("Head: {:?}", &data[0..4]);
        // println!("bb: {:?}", b"VISIO");
        let s = String::from_utf8_lossy(&data[0..18]);

        if s != "Visio (TM) Drawing" {
            panic!("This is not vsd file");
        }
        let version = u8::from_le_bytes([data[0x1A]]);
        println!("Visio format version: {}", version);

        let mut input: Cursor<Vec<u8>> = Cursor::new(data.clone());

        input.seek(SeekFrom::Start(0x24)).unwrap();

        // let _ = internal_stream.seek(0x24, SeekType::RVNG_SEEK_SET);

        let mut pointer = Pointer::new();

        pointer.read(&mut input);

        let mut shift = 0;
        let compressed = (pointer.format & 2) == 2;
        if compressed {
            shift = 4;
        }

        input
            .seek(SeekFrom::Start(pointer.offset.try_into().unwrap()))
            .unwrap();

        println!("compressed {}", compressed);

        println!("pointer {:?}", pointer);

        let mut internal_stream: VSDInternalStream =
            VSDInternalStream::new(&mut input, pointer.length.try_into().unwrap(), false).unwrap();

        handle_streams(&mut internal_stream, VSD_TRAILER_STREAM, shift);

        // fn read_null_terminated(bytes: &[u8]) -> &str {
        //     match bytes.iter().position(|&b| b == 0) {
        //         Some(pos) => std::str::from_utf8(&bytes[..pos]).unwrap_or(""),
        //         None => std::str::from_utf8(bytes).unwrap_or("."),
        //     }
        // }

        // println!("{}", read_null_terminated(&data));

        // let s = &data.iter().map(|&b| b as char).collect::<String>();
        // println!("{}", s);
        // Проверка сигнатуры (пример)
        // if data.len() < 8 || &data[0..4] != b"VISI" {
        //     return Err("Invalid Visio file signature".into());
        // }

        Ok(())
    }

    fn check_is_vsd(data_bytes: &[u8]) -> bool {
        // Магическое число OLE Compound Document
        if &data_bytes[..4] != b"\xD0\xCF\x11\xE0" {
            println!("Это не vsd!");
            false
        } else {
            println!("Похоже на формат Visio (OLE Compound Document).");
            true
        }
    }

    fn handle_streams(input: &mut VSDInternalStream, ptr_type: u8, shift: u32) {
        let (list_size, pointer_count) = read_pointer_info(input, shift);
        println!("list_size {} pointer_count {}", list_size, pointer_count)
    }

    pub fn read_pointer_info<R: Read + Seek>(input: &mut R, shift: u32) -> (u32, i32) {
        println!(
            "{} {}",
            "VSDParser::readPointerInfo",
            input.stream_position().unwrap()
        );
        println!("shift {}", shift);
        // Переходим к позиции shift
        let _ = input.seek(SeekFrom::Start(shift as u64));

        // Читаем смещение
        let offset = input.read_u32::<LittleEndian>().unwrap();

        println!("offset {}", offset);

        // Переходим к позиции offset + shift - 4
        let new_pos = (offset as u64) + (shift as u64) - 4;

        println!("new_pos {}", new_pos);
        let _ = input.seek(SeekFrom::Start(new_pos)).unwrap();

        // Читаем размер списка и количество указателей
        let list_size = match input.read_u32::<LittleEndian>() {
            Ok(va) => va,
            Err(_) => 0,
        };
        println!("list_size {}", list_size);
        let pointer_count = input.read_i32::<LittleEndian>().unwrap();

        // Пропускаем 4 байта
        let _ = input.seek(SeekFrom::Current(4));

        (list_size, pointer_count)
    }
}
