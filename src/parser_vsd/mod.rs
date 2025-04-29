pub mod read_vsd {

    use byteorder::ReadBytesExt;
    use std::io::{BufRead, Seek, SeekFrom};
    use std::{fs::File, io::BufReader, path::Path};

    #[derive(Debug, PartialEq)]
    pub struct Coordinate {
        pub x: i32,
        pub y: i32,
    }

    struct Pointer {
        pub type_num: u8,
        pub offset: u32,
        pub length: u32,
        pub format: u8,
        pub list_size: u32,
    }

    pub fn read_file(file_path: &Path) {
        match File::open(file_path) {
            Ok(file) => {
                println!("Файл успешно открыт.");

                let mut reader = BufReader::new(file);
                let data_bytes = reader.fill_buf().unwrap();
                println!("data_bytes {:?}", data_bytes.len());
                let is_vsd = check_is_vsd(data_bytes);

                if !is_vsd {
                    return;
                }

                reader.seek(SeekFrom::Start(26)).unwrap();

                let version = reader.read_u8();

                println!("Версия продукта {:?}", version.unwrap());

                reader.seek(SeekFrom::Start(0x24)).unwrap();

                let type_name = reader.read_u16::<byteorder::LittleEndian>();

                println!("type_name {:?}", type_name.unwrap() & 0x00ff);

                println!("stream_position {:?}", reader.stream_position().unwrap());

                let format = reader.read_u16::<byteorder::LittleEndian>();

                println!("format {:?}", format.unwrap());

                println!("stream_position {:?}", reader.stream_position().unwrap());

                let pos = reader.stream_position().unwrap();

                let _ = reader.seek(SeekFrom::Start(pos + 4));

                let offset = reader.read_u32::<byteorder::LittleEndian>().unwrap();

                println!("offset {:?}", offset);

                println!("stream_position {:?}", reader.stream_position().unwrap());

                let length = reader.read_u32::<byteorder::LittleEndian>();

                println!("length {:?}", length.unwrap());

                println!("stream_position {:?}", reader.stream_position().unwrap());

                let _ = reader.seek(SeekFrom::Start(offset.into()));

                println!("stream_position {:?}", reader.stream_position().unwrap());
            }
            Err(err) => {
                eprintln!(
                    "Ошибка открытия файла {:?}: {}",
                    file_path.file_name().unwrap().to_owned(),
                    err
                );
            }
        };
    }

    fn check_is_vsd(data_bytes: &[u8]) -> bool {
        if &data_bytes[..4] != b"\xD0\xCF\x11\xE0" {
            // Магическое число OLE Compound Document
            println!("Это не vsd!");
            false
        } else {
            println!("Похоже на формат Visio (OLE Compound Document).");
            true
        }
    }
}
