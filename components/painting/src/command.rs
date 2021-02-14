use super::color::Color;
use super::primitive::{Rect, RRect};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum DisplayCommand {
    FillRect(Rect, Color),
    FillRRect(RRect, Color),
    StrokeRect(Rect, Color),
}
