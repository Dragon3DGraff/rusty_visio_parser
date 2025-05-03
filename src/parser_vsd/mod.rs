pub mod read_vsd {

    use crate::helpers::VSDInternalStream;
    use crate::vsd_constants::object_types::VSD_TRAILER_STREAM;
    use byteorder::{LittleEndian, ReadBytesExt};
    use cfb::{self};
    use std::io::Cursor;
    use std::io::{Read, Seek, SeekFrom};
    use std::{fs::File, io::BufReader, path::Path};

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
        fn read(&mut self, stream: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn std::error::Error>> {
            self.type_name = stream.read_u32::<byteorder::LittleEndian>()?;
            let _ = stream.seek(SeekFrom::Current(4))?;
            self.format = stream.read_u32::<byteorder::LittleEndian>()?;
            self.offset = stream.read_u32::<byteorder::LittleEndian>()?;
            self.length = stream.read_u32::<byteorder::LittleEndian>()?;
            self.list_size = stream.read_u16::<byteorder::LittleEndian>()?;
            Ok(())
        }
    }

    pub fn read_file(file_path: &Path) {
        let mut cf = cfb::open_rw(file_path).unwrap();

        for entry in cf.read_root_storage() {
            println!("Found stream: {} {} bytes", entry.name(), entry.len());
        }
        match cf.open_stream("VisioDocument") {
            Err(err) => {
                println!("open_stream VisioDocument err: {}", err)
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
    }

    fn parse_visio_document(data: &Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        // println!("parse_visio_document {} bytes", data.len());

        let head_string = String::from_utf8_lossy(&data[0..18]);

        if head_string != "Visio (TM) Drawing" {
            panic!("This is not vsd file");
        }
        let version = u8::from_le_bytes([data[0x1A]]);
        println!("Visio format version: {}", version);

        let mut input: Cursor<Vec<u8>> = Cursor::new(data.clone());

        input.seek(SeekFrom::Start(0x24)).unwrap();

        let mut trailer_pointer = Pointer::new();

        let _ = trailer_pointer.read(&mut input);
        println!("trailer_pointer {:?}", trailer_pointer);

        let mut shift = 0;
        let compressed = (trailer_pointer.format & 2) == 2;

        println!("compressed: {}", compressed);

        if compressed {
            shift = 4;
        }

        input
            .seek(SeekFrom::Start(trailer_pointer.offset.try_into().unwrap()))
            .unwrap();

        let mut internal_stream: VSDInternalStream = VSDInternalStream::new(
            &mut input,
            trailer_pointer.length.try_into().unwrap(),
            compressed,
        )
        .unwrap();

        handle_streams(&mut internal_stream, VSD_TRAILER_STREAM, shift);

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

    fn handle_streams(input: &mut VSDInternalStream, ptr_type: u8, shift: u32) {
        let (list_size, pointer_count) = match read_pointer_info(input, shift) {
            Err(err) => {
                eprintln!("Ошибка handle_streams: {}", err);
                return ();
            }
            Ok(res) => res,
        };
        println!("list_size {} pointer_count {}", list_size, pointer_count)
    }

    pub fn read_pointer_info<R: Read + Seek>(
        input: &mut R,
        shift: u32,
    ) -> Result<(u32, u32), Box<dyn std::error::Error>> {
        println!(
            "{} {}",
            "VSDParser::readPointerInfo",
            input.stream_position().unwrap()
        );
        println!("shift {}", shift);
        // Переходим к позиции shift
        let _ = input.seek(SeekFrom::Start(shift as u64))?;

        // Читаем смещение
        let offset = input.read_u32::<LittleEndian>()?;

        println!("offset {}", offset);

        // Переходим к позиции offset + shift - 4
        let new_pos = (offset as u64) + (shift as u64) - 4;

        println!("new_pos {}", new_pos);
        let _ = input.seek(SeekFrom::Start(new_pos))?;

        // Читаем размер списка и количество указателей
        let list_size = match input.read_u32::<LittleEndian>() {
            Ok(va) => va,
            Err(_) => 0,
        };
        println!("list_size {}", list_size);

        let pointer_count = match input.read_u32::<LittleEndian>() {
            Ok(va) => va,
            Err(_) => 0,
        };

        // Пропускаем 4 байта
        let _ = input.seek(SeekFrom::Current(4));

        Ok((list_size, pointer_count))
    }
}
