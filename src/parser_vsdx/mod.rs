pub mod read_vsdx {
    use crate::get_masters_rel;
    use crate::get_metadata;
    use serde_json::to_string_pretty;
    use std::{collections::HashMap, fs, io::BufReader, path::Path};

    use serde::Serialize;

    #[derive(Serialize, Debug, Clone)]
    pub struct Diagram {
        pub rels: HashMap<String, HashMap<String, String>>,
        pub pages: Vec<Page>,
        pub masters_rel: HashMap<String, String>,
    }

    #[derive(Serialize, Debug, Clone)]
    pub struct Page {
        shapes: Vec<Shape>,
    }

    #[derive(Serialize, Debug, Clone)]
    pub struct Shape {}

    #[derive(Serialize, Debug, Clone)]
    pub struct Rel {
        id: String,
        type_url: String,
        target: String,
    }

    pub fn read_file(fname: &Path, out_dir: &Path) {
        let file = match fs::File::open(fname) {
            Ok(res) => res,
            Err(e) => {
                println!("Read file Error{e}");
                return;
            }
        };
        let reader = BufReader::new(file);

        let mut archive = zip::ZipArchive::new(reader).unwrap();
        let mut diagram = Diagram {
            pages: vec![],
            rels: HashMap::new(),
            masters_rel: HashMap::new(),
        };

        for i in 0..archive.len() {
            let mut file: zip::read::ZipFile<'_, BufReader<fs::File>> =
                archive.by_index(i).unwrap();
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

                if !&fname.ends_with(".xml") && !&fname.ends_with(".xml.rels") {
                    continue;
                }

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
}
