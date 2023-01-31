use std::rc::Rc;
use crate::dom::{NodeData, Payload};

pub struct Attribute<'a>(&'a str, &'a str);

pub fn get_node_by_attribute(result: &mut Option<&Rc<NodeData>>, source: &Rc<NodeData>, attribute: &Attribute) {
    if let Payload::Tag(tag) = source.get_payload() {
        if let Some(attribute_value) = tag.get_attribute_value(attribute.0) {
            if &attribute_value == attribute.1 {
                result.replace(source);
                return;
            }
        }
    }

    for child in source.get_children() {
        get_node_by_attribute(result, child, attribute);
        if result.is_some() {
            break;
        }
    }
}

pub fn get_node_by_name(result: &mut Option<&Rc<NodeData>>, source: &Rc<NodeData>, tag_name: &str) {
    if let Payload::Tag(tag) = source.get_payload() {
        if tag.get_name() == tag_name {
            result.replace(source);
            return;
        }
    }

    for child in source.get_children() {
        get_node_by_name(result, child, tag_name);
        if result.is_some() {
            break;
        }
    }
}


pub fn get_nodes_by_attribute(result: &mut Vec<&Rc<NodeData>>, source: &Rc<NodeData>, attribute: &Attribute) {
    if let Payload::Tag(tag) = source.get_payload() {
        if let Some(attribute_value) = tag.get_attribute_value(attribute.0) {
            if &attribute_value == attribute.1 {
                result.push(source);
            }
        }
    }

    for child in source.get_children() {
        get_nodes_by_attribute(result, child, attribute);
    }
}

pub fn get_nodes_by_name(result: &mut Vec<&Rc<NodeData>>, source: &Rc<NodeData>, tag_name: &str) {
    if let Payload::Tag(tag) = source.get_payload() {
        if tag.get_name() == tag_name {
            result.push(source);
        }
    }

    for child in source.get_children() {
        get_nodes_by_name(result, child, tag_name);
    }
}

pub fn get_first_child(node: &Rc<NodeData>) -> Option<&Rc<NodeData>> {
    let children = node.get_children();
    match children.len() {
        0 => None,
        _ => Some(&children[0]),
    }
}
