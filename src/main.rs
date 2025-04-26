use std::collections::HashMap;
use std::fs;
use std::io::BufReader;

use helpers::get_masters_rel;
use serde::Serialize;
use serde_json::to_string_pretty;

mod get_metadata;
mod helpers;
mod parse_vsd;

fn main() {
    std::process::exit(real_main());
}
#[derive(Serialize, Debug, Clone)]
pub struct Diagram {
    rels: HashMap<String, HashMap<String, String>>,
    pages: Vec<Page>,
    masters_rel: HashMap<String, String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Rel {
    id: String,
    type_url: String,
    target: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct Page {
    shapes: Vec<Shape>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Shape {}

fn real_main() -> i32 {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 {
        println!("need more args");
        return 1;
    }
    let fname = std::path::Path::new(&*args[1]);
    let out_dir = std::path::Path::new(&*args[2]);

    if (&*args[1])
        .to_lowercase()
        .ends_with(String::from(".vsd").as_str())
    {
        parse_vsd::read_vsd::read_file(fname);
        return 0;
    }

    if !(&*args[1])
        .to_lowercase()
        .ends_with(String::from(".vsdx").as_str())
    {
        println!("Only VSDX supports");
        return 1;
    }

    let file = match fs::File::open(fname) {
        Ok(res) => res,
        Err(e) => {
            println!("{e}");
            return 1;
        }
    };

    let reader = BufReader::new(file);

    let mut archive = zip::ZipArchive::new(reader).unwrap();
    // let mut archive = match zip::ZipArchive::(reader) {
    //     Err(e) => {
    //         println!("{e}");
    //         return 1;
    //     }
    //     Ok(res) => res,
    // };

    let mut diagram = Diagram {
        pages: vec![],
        rels: HashMap::new(),
        masters_rel: HashMap::new(),
    };

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
            // println!("{:?}", &mangled_name.starts_with(base));
            let hash_elements = get_metadata::encoding(&mut file, &mut diagram);

            if fname == "masters.xml" {
                get_masters_rel(&hash_elements, &mut diagram);
            }

            let json_str = match to_string_pretty(&hash_elements) {
                Ok(res) => res,
                Err(_) => {
                    print!("Err json");
                    "No data".to_string()
                }
            };
            let res_folder = out_dir.join(std::path::Path::new(&("jsons")));
            let _ = fs::create_dir(&res_folder);

            fs::write(
                res_folder.join(std::path::Path::new(&(fname.to_owned() + ".json"))),
                json_str,
            )
            .expect("Unable to write file");
        }

        let json_str = match to_string_pretty(&diagram) {
            Ok(res) => res,
            Err(_) => {
                print!("Err json");
                "No data".to_string()
            }
        };
        fs::write(
            out_dir.join(std::path::Path::new(&("Diagram.json"))),
            json_str,
        )
        .expect("Unable to write file");
    }

    0
}
