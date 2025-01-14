use super::ExpandOutput;
use crate::property::Property;
use crate::value::Value;
use css::parser::structs::ComponentValue;

pub fn expand_margin(values: &[&[ComponentValue]]) -> ExpandOutput {
    if values.len() == 1 {
        // this is a single value
        let value = Value::parse(&Property::MarginTop, values[0]);

        if value.is_none() {
            return None;
        }

        return Some(vec![
            (Property::MarginTop, value.clone()),
            (Property::MarginRight, value.clone()),
            (Property::MarginBottom, value.clone()),
            (Property::MarginLeft, value),
        ]);
    }

    if values.len() == 2 {
        // this is margin (x, y)
        let margin_y = Value::parse(&Property::MarginRight, values[0]);
        let margin_x = Value::parse(&Property::MarginRight, values[1]);

        if margin_x.is_none() || margin_y.is_none() {
            return None;
        }

        return Some(vec![
            (Property::MarginTop, margin_y.clone()),
            (Property::MarginRight, margin_x.clone()),
            (Property::MarginBottom, margin_y),
            (Property::MarginLeft, margin_x),
        ]);
    }

    if values.len() <= 4 {
        let margin_top = Value::parse(&Property::MarginRight, values[0]);
        let margin_right = Value::parse(&Property::MarginRight, values[1]);
        let margin_bottom = Value::parse(&Property::MarginRight, values[2]);

        if margin_top.is_none() || margin_right.is_none() || margin_bottom.is_none() {
            return None;
        }

        if values.len() == 3 {
            return Some(vec![
                (Property::MarginTop, margin_top),
                (Property::MarginRight, margin_right),
                (Property::MarginBottom, margin_bottom),
                (Property::MarginLeft, None),
            ]);
        }

        let margin_left = Value::parse(&Property::MarginRight, values[3]);

        if margin_left.is_none() {
            return None;
        }

        return Some(vec![
            (Property::MarginTop, margin_top),
            (Property::MarginRight, margin_right),
            (Property::MarginBottom, margin_bottom),
            (Property::MarginLeft, margin_left),
        ]);
    }

    None
}
