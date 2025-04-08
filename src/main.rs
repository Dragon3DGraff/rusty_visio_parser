use std::fs;
use std::io::BufReader;

use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

mod get_metadata;

fn main() {
    std::process::exit(real_main());
}

fn real_main() -> i32 {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 {
        println!("need out dir");
        return 1;
    }
    let fname = std::path::Path::new(&*args[1]);
    let out_dir = std::path::Path::new(&*args[2]);
    let file = fs::File::open(fname).unwrap();
    let reader = BufReader::new(file);

    let mut archive = zip::ZipArchive::new(reader).unwrap();

    #[derive(Serialize, Deserialize, Debug)]
    struct Shape {
        cells: Vec<Cell>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct Cell {
        name: String,
        value: String,
    }

    for i in 0..archive.len() {
        let mut file: zip::read::ZipFile<'_, BufReader<fs::File>> = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => {
                println!("Entry {} has a suspicious path", file.name());
                continue;
            }
        };

        {
            let comment = file.comment();
            if !comment.is_empty() {
                println!("Entry {i} comment: {comment}");
            }
        }

        if file.is_dir() {
            println!(
                "Entry {} is a directory with name \"{}\"",
                i,
                outpath.display()
            );
        } else {
            let mangled_name = &file.mangled_name();
            let xmlfile: BufReader<&mut zip::read::ZipFile<'_, BufReader<fs::File>>> =
                BufReader::new(&mut file);

            let f_name_p = match mangled_name.file_name() {
                Some(name) => name,
                None => {
                    println!("No name");
                    continue;
                }
            };

            let fname = match f_name_p.to_str() {
                Some(name) => name,
                None => {
                    println!("No name");
                    continue;
                }
            };

            // if fname == "page2.xml" {
            // println!("{}", fname);
            let hash_elements = get_metadata::encoding(xmlfile);

            let json_str = match to_string_pretty(&hash_elements) {
                Ok(res) => res,
                Err(_) => {
                    print!("Err json");
                    "No data".to_string()
                }
            };

            fs::write(
                out_dir.join(std::path::Path::new(&(fname.to_owned() + ".json"))),
                json_str,
            )
            .expect("Unable to write file");
            // }
        }
    }

    0
}
