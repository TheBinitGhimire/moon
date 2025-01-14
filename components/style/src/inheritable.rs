use super::property::Property;
use std::collections::HashSet;

lazy_static! {
    pub static ref INHERITABLES: HashSet<Property> = {
        let mut set = HashSet::new();
        set.insert(Property::Color);
        set.insert(Property::FontSize);
        set
    };
}
