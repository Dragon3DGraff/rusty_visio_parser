use std::io::BufReader;
use std::{collections::HashMap, fs};

use xml::reader::{EventReader, XmlEvent};

// use quickxml_to_serde::{Config, NullValue, xml_string_to_json};
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

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

    #[derive(Serialize, Debug, Clone)]
    struct Element {
        inner_id: i64,
        name: String,
        attrs: HashMap<String, String>,
        // level: i64,
        // children: Vec< Element>,
        children: Vec< i64>,
        parent: i64, // namespaces: Vec<Cell>,
    }

    impl Element {
        fn add_child(&mut self, elem_id:  i64 ) {
            self.children.push(elem_id);
        }
    }

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
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
            // let doc: Element = Element {
            //     inner_id: 0,
            //     name: "Document".to_string(),
            //     attrs: HashMap::new(),
            //     children: Vec::new(),
            //     parent: 0,
            //     // namespaces: Vec::new(),
            // };

            let xmlfile = BufReader::new(&mut file);

            // let conf = Config::new_with_defaults();
            // let json = xml_string_to_json(file.re, &conf);

            let parser = EventReader::new(xmlfile);
            // let mut depth = 0;
            let mut number_count = 0;
            // let mut stack: Vec<Element> = Vec::new();
            let mut hash_elements: HashMap<i64, Element> = HashMap::new();
            // prev_el.push(doc);
            // let mut stack = vec![Value::Null]; // Начальная точка стека
            let mut stack_numbers: Vec<i64> = Vec::new();
            for e in parser {
                match e {
                    Ok(XmlEvent::StartDocument { .. }) => {
                        // println!("version {}", version);
                        // println!("encoding {}",encoding);
                        // println!("standalone {:?}",standalone);
                    }
                    Ok(XmlEvent::StartElement {
                        name,
                        attributes,
                        ..
                        // namespace,
                    }) => {
                        // depth += 1;
                        number_count += 1;
                        // print!("namespace {:?}", namespace.0);
                        let tag_name = name.local_name;
                        // let attrs = attributes
                        //     .into_iter()
                        //     .map(|attr| Cell {
                        //         name: attr.to_owned().name.local_name,
                        //         value: attr.to_owned().value,
                        //     })
                        //     .collect::<Vec<_>>();

                        let mut attrs = HashMap::new();
                        for attr in attributes {
                            let key = attr.to_owned().name.local_name;
                            let value = attr.to_owned().value;
                            attrs.insert(key.to_owned(), value.to_owned());
                        }

                        let parent = match stack_numbers.last() {
                            Some(parent) => parent,
                            None => &0,
                        };

                        let element = Element {
                            inner_id: number_count,
                            name: tag_name,
                            attrs,
                            children: vec![],
                            parent: *parent,
                            // namespaces: namespace
                            //     .0
                            //     .iter()
                            //     .map(|(key, val)| Cell {
                            //         name: key.to_owned(),
                            //         value: val.to_owned(),
                            //     })
                            //     .collect::<Vec<_>>(),
                        };

                        hash_elements.insert(number_count, element.clone());
                        hash_elements.entry(*parent).and_modify(|e: &mut Element| { e.add_child(element.inner_id); });

                        // match hash_elements.get_mut(parent) {
                        //     Some(el)=>{
                        //         match hash_elements.get(&number_count) {
                        //             Some(cur_el) =>{
                        //                 el.add_child(&cur_el);
                        //             }
                        //             None => ()
                                    
                        //         }; 
                               
                        //     }
                        //     None => ()
                        // }; 
                        

                        // match prev_el.last_mut().unwrap() {
                        //     Some(prev) => {
                        //         // prev.add_child(&elem);
                        //     }
                        //     None => {
                        //         println!("No last item")
                        //     }
                        // };
                        // if let Some(ref mut arr) = stack.last_mut().unwrap() {
                        //     arr.children.push(&elem);
                        // } else {
                        //     return Err("Invalid state".into());
                        // }
                        // if !stack.is_empty() {
                        //     // Добавляем элемент в последний объект стека
                        //     if let Some(Value::Array(obj)) =
                        //         stack.last_mut().and_then(|v: &mut Value| v.as_object_mut())
                        //     {
                        //         obj.insert(name.local_name.to_string(), json!(element));
                        //     }
                        // } else {
                        //     // Начинаем новый корневой объект
                        //     stack.push(json!(element));
                        // }
                        // if let Some(Value::Array(ref mut arr)) = stack.last_mut().unwrap() {
                        //     arr.push(serde_json::to_value(current_element)?);
                        // } else {
                        //     return Err("Invalid state".into());
                        // }

                        // stack.push(Value::Array(vec![]));
                        // stack.push(element);
                        stack_numbers.push(number_count);
                        // prev_el.push(elem);
                        // doc.add_child(elem);

                        //     println!("attributes {:?}", attrs);
                        //     // println!("{}", cell_name);

                        //     // let cell = Cell {
                        //     //     name: cell_name,
                        //     //     value: cell_val,
                        //     // };

                        //     // let json_str = match serde_json::to_string(&attrs) {
                        //     //     Ok(res) => res,
                        //     //     Err(_) => {
                        //     //         print!("Err json");
                        //     //         "No data".to_string()
                        //     //     }
                        //     // };

                        //     // txt.push(json_str);
                        //     shapes.push(Shape {
                        //         cells: attrs
                        //     });
                        // } else {
                        //     txt.push(tag_name);
                        // }
                    }
                    Ok(XmlEvent::EndElement { .. }) => {
                        // depth -= 1;
                        stack_numbers.pop();
                    }
                    Ok(XmlEvent::EndDocument { .. }) => {}
                    Ok(XmlEvent::Characters(text)) => {
                        number_count += 1;
                        let parent = match stack_numbers.last() {
                            Some(parent) => parent,
                            None => &0,
                        };
                        // doc.add_child(Element {
                        //     name: text,
                        //     attrs: Vec::new(),
                        //     level: depth,
                        //     children: Vec::new(),
                        //  //   namespaces: Vec::new(),
                        // });
                        let mut attrs = HashMap::new();

                        attrs.insert("Text".to_string(), text);

                        let  element =Element {
                            inner_id: number_count,
                            name: "Characters".to_string(),
                            attrs,
                            children: Vec::new(),
                            parent: *parent
                        };

                        hash_elements.insert(number_count, element.to_owned());

                                //  stack.push(element);

                    }
                    Err(e) => {
                        eprintln!("Error: {e} {}", outpath.display());
                        break;
                    }
                    // There's more: https://docs.rs/xml-rs/latest/xml/reader/enum.XmlEvent.html
                    _ => {}
                }
            }

            let mangled_name = file.mangled_name();
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
        }
    }

    0
}
