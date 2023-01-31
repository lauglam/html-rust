use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};

mod payload;

pub use payload::Payload;
pub use payload::Tag;
pub use payload::Text;

type NodeDataRef = Rc<NodeData>;
type WeakNodeDataRef = Weak<NodeData>;

/// Parent relationship is one of non-ownership.
/// This is not a `RefCell<NodeDataRef<T>>` which would cause memory leak.
type Parent = RefCell<WeakNodeDataRef>;

/// Children relationship is one of ownership.
type Child = NodeDataRef;
type Children = RefCell<Vec<Child>>;

/// This struct holds underlying data. It shouldn't be created directly, instead use:
/// [`Node`](struct@Node).
///
/// ```text
/// NodeData
///  | | |
///  | | +- payload: T ---------------------------------------+
///  | |                                                    |
///  | |                                        Simple ownership of payload
///  | |
///  | +-- parent: RefCell<WeakNodeDataRef<T>> --------+
///  |                                            |
///  |                 This describes a non-ownership relationship.
///  |                 When a node is dropped, its parent will not be dropped.
///  |
///  +---- children: RefCell<Vec<Child<T>>> ---+
///                                           |
///                 This describes an ownership relationship.
///                 When a node is dropped its children will be dropped as well.
/// ```
#[derive(Debug, Clone)]
pub struct NodeData {
    payload: Payload,
    parent: Parent,
    children: Children,
}

impl PartialEq for NodeData {
    fn eq(&self, other: &Self) -> bool {
        self.payload == other.payload
            && self.children == other.children
    }
}

impl NodeData {
    pub fn get_payload(&self) -> &Payload {
        &self.payload
    }

    pub fn get_children(&self) -> &Vec<NodeDataRef> {
        self.children.borrow().as_ref()
    }

    pub fn get_parent(&self) -> Option<NodeDataRef> {
        let parent_weak = self.parent.borrow();
        match parent_weak.upgrade() {
            Some(parent_rc_ref) => Some(parent_rc_ref),
            None => None,
        }
    }

    pub fn has_parent(&self) -> bool {
        self.get_parent().is_some()
    }
}

/// This struct is used to own a [`NodeData`] inside an [`Rc`]. The [`Rc`]
/// can be shared, so that it can have multiple owners. It does not have
/// getter methods for [`NodeData`]'s properties, instead it implements the
/// `Deref` trait to allow it to be used as a [`NodeData`].
///
/// # Shared ownership
///
/// After an instance of this struct is created and it's internal reference is
/// cloned (and given to another) dropping this instance will not drop the cloned
/// internal reference.
///
/// ```text
/// Node { rc_ref: Rc<NodeData> }
///    ▲                 ▲
///    │                 │
///    │      This rc ref owns the
///    │      `NodeData` & is shared
///    │
///    1. Has methods to manipulate nodes and their children.
///
///    2. When it is dropped, if there are other `Arc`s (shared via
///       `get_copy_of_internal_arc()`) pointing to the same underlying
///       `NodeData`, then the `NodeData` will not be dropped.
///
///    3. This struct is necessary in order for `add_child_and_update_its_parent`
///       to work. Some pointers need to be swapped between 2 nodes for this work
///       (and one of these pointers is a weak one). It is not possible to do this
///       using two `NodeData` objects, without wrapping them in `Arc`s.
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct Node {
    rc_ref: NodeDataRef,
}

impl Deref for Node {
    type Target = NodeData;

    fn deref(&self) -> &Self::Target {
        &self.rc_ref
    }
}

impl Node {
    pub fn new(payload: Payload) -> Node {
        let new_node = NodeData {
            payload,
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(Vec::new()),
        };

        let rc_ref = Rc::new(new_node);
        Node { rc_ref }
    }

    pub fn get_copy_of_internal_arc(&self) -> NodeDataRef {
        Rc::clone(&self.rc_ref)
    }

    pub fn add_child_and_update_parent(&self, child: &Node) {
        {
            let mut children = self.children.borrow_mut();
            children.push(child.get_copy_of_internal_arc());
        }

        {
            let mut childs_parent = child.parent.borrow_mut();
            *childs_parent = Rc::downgrade(&self.get_copy_of_internal_arc());
        }
    }

    pub fn create_and_add_child(&self, payload: Payload) -> NodeDataRef {
        let new_child = Node::new(payload);
        self.add_child_and_update_parent(&new_child);
        new_child.get_copy_of_internal_arc()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_test() {
        let child = Node::new(
            Payload::Text(
                String::from("Hello, world!")
            ));
        let parent = Node::new(
            Payload::Tag(
                Tag::new("li")
            ));

        parent.add_child_and_update_parent(&child);

        assert!(child.has_parent());
    }

    #[test]
    fn copy_test() {
        let node = Node::new(
            Payload::Tag(
                Tag::new("li")
            ));

        let copy_node = node.get_copy_of_internal_arc();
        assert_eq!(node.rc_ref, copy_node)
    }
}
