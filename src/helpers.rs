use std::collections::HashMap;

use xml::attribute::OwnedAttribute;

pub fn attrs_to_hashmap(attributes: &Vec<OwnedAttribute>) -> HashMap<String, String> {
    let mut attrs: HashMap<String, String> = HashMap::new();

    for attr in attributes {
        let key = attr.to_owned().name.local_name;
        let value = attr.to_owned().value;
        attrs.insert(key.to_owned(), value.to_owned());
    }

    attrs
}
