use std::collections::HashMap;

pub type Text = String;

#[derive(Debug, PartialEq, Clone)]
pub struct Tag {
    name: String,
    attributes: Option<HashMap<String, String>>,
    // A flag that represents a tag whether is self-closing. <tag />
    self_closing: bool,
    // A flag that represents a tag whether is the closed one. </ tag>
    terminator: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Payload {
    Tag(Tag),
    Text(Text),
    Comment(Text),
}

impl Tag {
    pub fn new(name: &str) -> Tag {
        Tag {
            name: String::from(name),
            attributes: None,
            self_closing: false,
            terminator: false,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_attributes(&mut self, attributes: HashMap<String, String>) {
        self.attributes = Some(attributes);
    }

    pub fn get_attributes(&self) -> Option<&HashMap<String, String>> {
        self.attributes.as_ref()
    }

    pub fn set_attribute(&mut self, attribute: &str, value: &str) {
        match self.attributes.as_mut() {
            Some(attributes) => {
                attributes.insert(String::from(attribute), String::from(value));
            }
            None => {
                let mut attributes = HashMap::new();
                attributes.insert(String::from(attribute), String::from(value));
                self.attributes = Some(attributes);
            }
        }
    }

    pub fn get_attribute_value(&self, attribute_name: &str) -> Option<String> {
        if let Some(attributes) = &self.attributes {
            if let Some(v) = attributes.get(attribute_name) {
                return Some(v.to_string());
            }
        }

        None
    }

    pub fn set_terminator(&mut self, b: bool) {
        self.terminator = b;
    }

    pub fn is_terminator(&self) -> bool {
        self.terminator
    }
    pub fn set_self_closing(&mut self, b: bool) {
        self.self_closing = b;
    }

    pub fn is_self_closing(&self) -> bool {
        self.self_closing
    }
}
