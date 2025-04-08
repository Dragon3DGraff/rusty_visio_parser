use serde::Serialize;
use std::io::BufReader;
use std::{collections::HashMap, fs};
use xml::reader::{EventReader, XmlEvent};

#[derive(Serialize, Debug, Clone)]
pub struct Element {
    inner_id: i64,
    name: String,
    attrs: HashMap<String, String>,
    params: HashMap<String, HashMap<String, String>>,
    children: Vec<i64>,
    parent: i64,
}

impl Element {
    fn add_child(&mut self, elem_id: i64) {
        self.children.push(elem_id);
    }
}

pub fn encoding(
    xmlfile: BufReader<&mut zip::read::ZipFile<'_, BufReader<fs::File>>>,
) -> HashMap<i64, Element> {
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
                number_count += 1;
                let tag_name = name.local_name;

                let parent = match stack_numbers.last() {
                    Some(parent) => parent,
                    None => &0,
                };

                let mut attrs = HashMap::new();

                for attr in &attributes {
                    let key = attr.to_owned().name.local_name;
                    let value = attr.to_owned().value;
                    attrs.insert(key.to_owned(), value.to_owned());
                }
                let element = Element {
                    inner_id: number_count,
                    name: tag_name.to_owned(),
                    attrs,
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
                        // let mut params = HashMap::new();
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
    hash_elements
}
