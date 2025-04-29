use std::collections::HashMap;

use xml::attribute::OwnedAttribute;

use crate::{parser_vsdx::get_metadata::Element, parser_vsdx::read_vsdx::Diagram};

pub fn attrs_to_hashmap(attributes: &Vec<OwnedAttribute>) -> HashMap<String, String> {
    let mut attrs: HashMap<String, String> = HashMap::new();

    for attr in attributes {
        let key = attr.to_owned().name.local_name;
        let value = attr.to_owned().value;
        attrs.insert(key.to_owned(), value.to_owned());
    }

    attrs
}

pub fn get_masters_rel(hash_elements: &HashMap<i64, Element>, diagram: &mut Diagram) {
    for (id, el) in hash_elements {
        if el.name == "Master" {
            for attr_id in &el.children {
                let child = hash_elements.get(&attr_id).unwrap();

                if child.name == "Rel" {
                    let rel_id = child.attrs.get("id").unwrap();
                    diagram
                        .masters_rel
                        .insert(id.to_string(), rel_id.to_owned());
                }
            }
        }
    }
}
