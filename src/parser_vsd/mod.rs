pub mod read_vsd {

    use crate::VSDInternalStream::VSDInternalStream;
    use crate::VSDParser::VSDParser;
    use crate::vsd_constants::object_types::{
        VSD_FONTFACES, VSD_NAME_LIST2, VSD_NAMEIDX, VSD_NAMEIDX123, VSD_SHAPE_GROUP,
        VSD_SHAPE_SHAPE, VSD_TRAILER_STREAM,
    };
    use byteorder::{LittleEndian, ReadBytesExt};
    use cfb::{self};
    use std::collections::{HashMap, HashSet};
    use std::io::Cursor;
    use std::io::{Read, Seek, SeekFrom};
    use std::{fs::File, io::BufReader, path::Path};

    #[derive(Debug, PartialEq)]
    pub struct Coordinate {
        pub x: i32,
        pub y: i32,
    }

    struct SectionHeader {
        type_id: u16, // Тип секции (например, 0x0015 для страницы)
        unused: u16,  // Резерв (обычно 0)
        size: u16,    // Размер данных секции (без заголовка)
    }

    impl SectionHeader {
        fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
            Ok(SectionHeader {
                type_id: reader.read_u16::<LittleEndian>()?,
                unused: reader.read_u16::<LittleEndian>()?,
                size: reader.read_u16::<LittleEndian>()?,
            })
        }
    }

    pub fn read_file(file_path: &Path) {
        let mut cf = cfb::open_rw(file_path).unwrap();

        for entry in cf.read_root_storage() {
            println!(
                "Found stream: {} {} bytes, is_root {}, path {:?}",
                entry.name(),
                entry.len(),
                entry.is_root(),
                entry.path()
            );
        }
        match cf.open_stream("VisioDocument") {
            Err(err) => {
                println!("open_stream VisioDocument err: {}", err)
            }
            Ok(mut stream) => {
                println!("open_stream VisioDocument");
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
    }

    fn parse_visio_document(data: &Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        // println!("parse_visio_document {} bytes", data.len());

        let head_string = String::from_utf8_lossy(&data[0..18]);

        if head_string != "Visio (TM) Drawing" {
            panic!("This is not vsd file");
        }
        let version = u8::from_le_bytes([data[0x1A]]);

        println!("Visio format version: {}", version);

        if version != 11 {
            panic!("Supported only version 11");
        }

        let mut input: Cursor<Vec<u8>> = Cursor::new(data.to_owned());

        // println!("len {} ", data.len());
        // let mut i = 0;
        // for by in data {
        //     input.seek(SeekFrom::Start(i)).unwrap();
        //     if i + 1 == data.len() as u64 {
        //         continue;
        //     }
        //     let val = input.read_u16::<LittleEndian>().unwrap();
        //     // print!(" {} ", val);
        //     if val == 0x0015 {
        //         println!(" Нашел ");
        //         // println!("by= {} ", by);
        //         // println!(" 0x{:04X} ", by);
        //         println!("i= {} ", i);
        //     }
        //     i += 1;
        // }

        // input.seek(SeekFrom::Start(3845))?;
        // while input.position() < data.len() as u64 {
        //     let header = SectionHeader::read(&mut input)?;
        //     println!(
        //         "Секция 0x{:04X}, размер={} байт",
        //         header.type_id, header.size
        //     );

        //     // Читаем данные секции (пример для Page)
        //     if header.type_id == 0x0015 {
        //         // read_page_section(&mut cursor, header.size)?;
        //     }

        //     // Пропускаем оставшиеся данные секции, если не обрабатываем
        //     input.seek(SeekFrom::Current(header.size as i64))?;
        // }

        let mut vsd_parser = VSDParser::new(data.to_owned());

        vsd_parser.parse_main();

        // input.seek(SeekFrom::Start(0x24)).unwrap();

        // let mut trailer_pointer = Pointer::new();

        // let _ = trailer_pointer.read(&mut input);

        // let mut shift = 0;
        // let compressed = (trailer_pointer.format & 2) == 2;

        // println!("compressed: {}", compressed);

        // if compressed {
        //     shift = 4;
        // }

        // input
        //     .seek(SeekFrom::Start(trailer_pointer.offset.try_into().unwrap()))
        //     .unwrap();

        // let mut internal_stream: VSDInternalStream = VSDInternalStream::new(
        //     &mut input,
        //     trailer_pointer.length.try_into().unwrap(),
        //     compressed,
        // )
        // .unwrap();

        // handle_streams(&mut internal_stream, VSD_TRAILER_STREAM, shift);

        Ok(())
    }

    pub fn check_is_vsd(file_path: &Path) -> bool {
        match File::open(file_path) {
            Ok(file) => {
                println!("Файл успешно открыт.");
                let mut reader: BufReader<File> = BufReader::new(file);
                let mut buf = vec![0; 8];
                reader
                    .read_exact(&mut buf)
                    .expect("Не удалось прочитать заголовок файла");
                // Магическое число OLE Compound Document
                if &buf[..4] != b"\xD0\xCF\x11\xE0" {
                    println!("Это не vsd!");
                    return false;
                } else {
                    println!("Похоже на формат Visio (OLE Compound Document).");
                    return true;
                }
            }
            Err(err) => {
                eprintln!(
                    "Ошибка открытия файла {:?}: {}",
                    file_path.file_name().unwrap().to_owned(),
                    err
                );
                return false;
            }
        };
    }

    // fn handle_streams(input: &mut VSDInternalStream, ptr_type: u8, shift: u32) {
    //     let mut pointer_order = Vec::new();
    //     let mut ptr_list = HashMap::new();
    //     let mut font_faces = HashMap::new();
    //     let mut name_list = HashMap::new();
    //     let mut name_idx = HashMap::new();

    //     let (mut list_size, pointer_count) = match read_pointer_info(input, shift) {
    //         Err(err) => {
    //             eprintln!("Ошибка handle_streams: {}", err);
    //             return ();
    //         }
    //         Ok(res) => res,
    //     };
    //     println!("list_size {} pointer_count {}", list_size, pointer_count);

    //     let mut input_cursor: Cursor<Vec<u8>> = Cursor::new(input.buffer.to_owned());

    //     for i in 0..pointer_count {
    //         let mut ptr = Pointer::new();
    //         let _ = ptr.read(&mut input_cursor);
    //         if ptr.type_name == 0 {
    //             continue;
    //         }

    //         match ptr.type_name as u8 {
    //             VSD_FONTFACES => {
    //                 font_faces.insert(i as u32, ptr);
    //             }
    //             VSD_NAME_LIST2 => {
    //                 name_list.insert(i as u32, ptr);
    //             }
    //             VSD_NAMEIDX | VSD_NAMEIDX123 => {
    //                 name_idx.insert(i as u32, ptr);
    //             }
    //             _ => {
    //                 ptr_list.insert(i as u32, ptr);
    //             }
    //         }
    //     }

    //     if list_size <= 1 {
    //         list_size = 0;
    //     }

    //     for _ in 0..list_size {
    //         pointer_order.push(input.read_u32::<byteorder::LittleEndian>());
    //     }

    //     // Process the streams in specific order
    //     for (idx, ptr) in name_list {
    //         println!("{}", idx);
    //         handle_stream(ptr, idx, level + 1, visited)?;
    //     }

    //     for (idx, ptr) in name_idx {
    //         println!("{}", idx);
    //         handle_stream(ptr, idx, level + 1, visited)?;
    //     }

    //     for (idx, ptr) in font_faces {
    //         println!("{}", idx);
    //         handle_stream(ptr, idx, level + 1, visited)?;
    //     }

    //     if !pointer_order.is_empty() {
    //         for j in pointer_order {
    //             println!("{:?}", ptr_list.get(&j.unwrap()))
    //             if let Some(ptr) = ptr_list.remove(&j) {
    //                 handle_stream(ptr, j, level + 1, visited)?;
    //             }
    //         }
    //     }

    //     for (idx, ptr) in ptr_list {
    //         println!("{}", idx)
    //         handle_stream(ptr, idx, level + 1, visited)?;
    //     }
    // }

    
}
