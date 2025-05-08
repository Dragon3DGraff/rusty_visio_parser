pub mod utils;
pub mod vsd_constants;
pub mod vsdinternal_stream;
// mod vsdparser;
pub mod VSDParser;

// use vsdparser::VSDParser;

use std::io::Read;
use std::{fs::File, io::BufReader, path::Path};

#[derive(Debug, PartialEq)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
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
    println!("parse_visio_document {} bytes", data.len());

    let head_string = String::from_utf8_lossy(&data[0..18]);

    if head_string != "Visio (TM) Drawing" {
        panic!("This is not vsd file");
    }
    let version = u8::from_le_bytes([data[0x1A]]);

    println!("Visio format version: {}", version);

    if version != 11 {
        panic!("Supported only version 11");
    }

    let mut vsd_parser = VSDParser::VSDParser::new(data.to_owned());

    vsd_parser.parse_main();

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
