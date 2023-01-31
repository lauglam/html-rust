use std::collections::HashMap;

pub type Text = String;

#[derive(Debug, PartialEq, Clone)]
pub struct Tag {
    name: String,
    attrs: Option<HashMap<String, String>>,
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
            attrs: None,
            self_closing: false,
            terminator: false,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_attrs(&mut self, attrs: HashMap<String, String>) {
        self.attrs = Some(attrs);
    }

    pub fn get_attrs(&self) -> Option<&HashMap<String, String>> {
        self.attrs.as_ref()
    }

    pub fn set_attr(&mut self, attr: &str, value: &str) {
        match self.attrs.as_mut() {
            Some(attrs) => {
                attrs.insert(String::from(attr), String::from(value));
            }
            None => {
                let mut attrs = HashMap::new();
                attrs.insert(String::from(attr), String::from(value));
                self.attrs = Some(attrs);
            }
        }
    }

    pub fn get_attr(&self, attr: &str) -> Option<String> {
        if let Some(attrs) = &self.attrs {
            if let Some(v) = attrs.get(attr) {
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
