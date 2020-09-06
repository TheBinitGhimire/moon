use super::node::Node;

pub struct CharacterData {
    pub node: Node,
    data: String
}

impl CharacterData {
    pub fn new(data: String) -> Self {
        Self {
            node: Node::new(),
            data
        }
    }
}
