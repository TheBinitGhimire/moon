use std::rc::Rc;

use crate::node::Node;

use super::node::NodeHooks;
use enum_dispatch::enum_dispatch;

mod html_anchor_element;
mod html_body_element;
mod html_div_element;
mod html_head_element;
mod html_html_element;
mod html_link_element;
mod html_title_element;
mod html_unknown_element;

pub use html_anchor_element::*;
pub use html_body_element::*;
pub use html_div_element::*;
pub use html_head_element::*;
pub use html_html_element::*;
pub use html_link_element::*;
pub use html_title_element::*;
pub use html_unknown_element::*;

#[enum_dispatch(ElementHooks, NodeHooks, ElementMethods)]
#[derive(Debug)]
pub enum ElementData {
    Anchor(HTMLAnchorElement),
    Body(HTMLBodyElement),
    Div(HTMLDivElement),
    Head(HTMLHeadElement),
    Html(HTMLHtmlElement),
    Title(HTMLTitleElement),
    Unknown(HTMLUnknownElement),
    Link(HTMLLinkElement),
}

#[enum_dispatch]
trait ElementHooks {
    #[allow(unused_variables)]
    fn on_attribute_change(&self, attr: &str, value: &str) {}
}

#[enum_dispatch]
pub trait ElementMethods {
    fn tag_name(&self) -> String {
        "unknown".to_string()
    }
}

impl ElementData {
    pub fn handle_attribute_change(&self, attr: &str, value: &str) {
        self.on_attribute_change(attr, value);
    }

    pub fn handle_on_inserted(&self, document: Rc<Node>) {
        self.on_inserted(document);
    }
}
