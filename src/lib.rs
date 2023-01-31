mod parser;
mod extent;

pub mod dom;

pub use parser::parse;

pub use extent::get_node_by_name;
pub use extent::get_node_by_attribute;

pub use extent::get_nodes_by_name;
pub use extent::get_nodes_by_attribute;

pub use extent::get_first_child;
