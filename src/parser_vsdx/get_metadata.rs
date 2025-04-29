use serde::Serialize;
use std::fs::File;
use std::io::BufReader;
use std::{collections::HashMap, fs};
use xml::reader::{EventReader, XmlEvent};
use zip::read::ZipFile;

use super::helpers::attrs_to_hashmap;
use crate::parser_vsdx::read_vsdx::Diagram;

#[derive(Serialize, Debug, Clone)]
pub struct Element {
    pub inner_id: i64,
    pub name: String,
    pub attrs: HashMap<String, String>,
    pub params: HashMap<String, HashMap<String, String>>,
    pub children: Vec<i64>,
    pub parent: i64,
}

impl Element {
    fn add_child(&mut self, elem_id: i64) {
        self.children.push(elem_id);
    }
}

pub fn encoding(
    file: &mut ZipFile<'_, BufReader<File>>,
    diagram: &mut Diagram,
) -> HashMap<i64, Element> {
    let mangled_name = &file.mangled_name();

    let fname = mangled_name.file_name().unwrap().to_str().unwrap();

    let mut temp_hash: HashMap<String, String> = HashMap::new();

    let xmlfile: BufReader<&mut zip::read::ZipFile<'_, BufReader<fs::File>>> = BufReader::new(file);

    let mut hash_elements: HashMap<i64, Element> = HashMap::new();

    let parser: EventReader<BufReader<&mut zip::read::ZipFile<'_, BufReader<fs::File>>>> =
        EventReader::new(xmlfile);
    let mut number_count = 0;
    let mut stack_numbers: Vec<i64> = Vec::new();

    for e in parser {
        match e {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                let tag_name = name.local_name;

                if fname.ends_with(".rels") {
                    if tag_name == "Relationship" {
                        let mut param_name = String::new();
                        let mut param_val = String::new();
                        for attr in &attributes {
                            let key = attr.to_owned().name.local_name;
                            let value = attr.to_owned().value;
                            if key == "Id" {
                                param_name = value
                            } else if key == "Target" {
                                param_val = value
                            }
                        }

                        temp_hash.insert(param_name, param_val);
                    }
                    continue;
                }

                number_count += 1;

                let parent = match stack_numbers.last() {
                    Some(parent) => parent,
                    None => &0,
                };

                let element = Element {
                    inner_id: number_count,
                    name: tag_name.to_owned(),
                    attrs: attrs_to_hashmap(&attributes),
                    children: vec![],
                    parent: *parent,
                    params: HashMap::new(),
                };

                if tag_name != "Cell" {
                    hash_elements.insert(number_count, element.clone());
                }
                // println!("{}", tag_name);

                hash_elements.entry(*parent).and_modify(|e: &mut Element| {
                    if tag_name == "Cell" {
                        let mut param_name = String::new();
                        let mut param_val = HashMap::new();
                        for attr in &attributes {
                            let key = attr.to_owned().name.local_name;
                            let value = attr.to_owned().value;
                            if key == "N" {
                                param_name = value
                            } else {
                                param_val.insert(key.to_owned(), value.to_owned());
                            }
                        }
                        e.params.insert(param_name, param_val);
                    } else {
                        e.add_child(element.inner_id);
                    }
                });

                stack_numbers.push(number_count);
            }
            Ok(XmlEvent::EndElement { .. }) => {
                stack_numbers.pop();
            }
            Ok(XmlEvent::EndDocument { .. }) => {}
            Ok(XmlEvent::Characters(text)) => {
                number_count += 1;
                let parent = match stack_numbers.last() {
                    Some(parent) => parent,
                    None => &0,
                };

                let mut attrs = HashMap::new();

                attrs.insert("Text".to_string(), text);

                let element = Element {
                    inner_id: number_count,
                    name: "Characters".to_string(),
                    attrs,
                    children: Vec::new(),
                    parent: *parent,
                    params: HashMap::new(),
                };

                hash_elements.insert(number_count, element.to_owned());
            }
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
            _ => {}
        }
    }
    if fname.ends_with(".rels") {
        diagram.rels.insert(fname.to_owned(), temp_hash);
    };
    hash_elements
}
