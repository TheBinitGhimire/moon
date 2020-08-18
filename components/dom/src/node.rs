use std::rc::{Weak, Rc};
use std::cell::RefCell;
use std::ops::{Deref};

#[derive(Debug, Clone)]
pub enum NodeType {
    Element = 1,
    Text = 3,
    CDataSection = 4,
    ProcessingInstruction = 7,
    Comment = 8,
    Document = 9,
    DocumentType = 10,
    DocumentFragment = 11
}

pub struct Node {
    node_type: NodeType,
    parent_node: Option<WeakNodeRef>,
    first_child: Option<NodeRef>,
    last_child: Option<WeakNodeRef>,
    next_sibling: Option<NodeRef>,
    prev_sibling: Option<WeakNodeRef>,
}

pub struct NodeRef(Rc<RefCell<Node>>);
pub struct WeakNodeRef(Weak<RefCell<Node>>);

impl Deref for NodeRef {
    type Target = RefCell<Node>;

    fn deref(&self) -> &RefCell<Node> {
        &*self.0
    }
}

impl Clone for WeakNodeRef {
    fn clone(&self) -> Self {
        Self(self.0.clone())        
    }
}

impl Clone for NodeRef {
    fn clone(&self) -> Self {
        Self(self.0.clone())        
    }
}

impl NodeRef {
    pub fn downgrade(self) -> WeakNodeRef {
        WeakNodeRef(Rc::downgrade(&self.0))
    }

    pub fn parent(&self) -> Option<NodeRef> {
        let ref_self = self.borrow();
        match &ref_self.parent_node {
            Some(node) => node.clone().upgrade(),
            _ => None
        }
    }

    pub fn next_sibling(&self) -> Option<NodeRef> {
        let ref_self = self.borrow();
        ref_self.next_sibling.clone()
    }

    pub fn prev_sibling(&self) -> Option<NodeRef> {
        let ref_self = self.borrow();
        match &ref_self.prev_sibling {
            Some(node) => node.clone().upgrade(),
            _ => None
        }
    }

    pub fn first_child(&self) -> Option<NodeRef> {
        let ref_self = self.borrow();
        ref_self.first_child.clone()
    }

    pub fn last_child(&self) -> Option<NodeRef> {
        let ref_self = self.borrow();
        match &ref_self.last_child {
            Some(node) => node.clone().upgrade(),
            _ => None
        }
    }

    pub fn node_type(&self) -> NodeType {
        let ref_self = self.borrow();
        ref_self.node_type.clone()
    }
}

impl WeakNodeRef {
    pub fn upgrade(self) -> Option<NodeRef> {
        match self.0.upgrade() {
            Some(node) => Some(NodeRef(node)),
            _ => None
        }
    }
}

#[cfg(test)]
mod test {
}