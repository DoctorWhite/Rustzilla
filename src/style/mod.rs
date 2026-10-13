pub mod color;
pub mod css;
pub mod values;

pub use color::Color;
pub use css::{SimpleSelector, Stylesheet};
pub use values::{DisplayType, Unit};

#[derive(Debug, Clone)]
pub struct StyleMap {
    pub display: DisplayType,
    pub width: Unit,
    pub height: Unit,
    pub color: Color,
    pub background_color: Color,
    pub margin_top: Unit,
    pub margin_bottom: Unit,
    pub margin_left: Unit,
    pub margin_right: Unit,
    pub flex_direction: String,
    pub padding_top: Unit,
    pub padding_bottom: Unit,
    pub padding_left: Unit,
    pub padding_right: Unit,
    pub font_size: Unit,
    pub font_weight: u16, // 400 = normal, 700 = bold
    pub text_align: String,
}
