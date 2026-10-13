use crate::style::DisplayType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlTag {
    Div,
    P,
    H1,
    H2,
    H3,
    Body,
    Html,
    Head,
    Span,
    A,
    B,
    Strong,
    Img,
    Other(String),
}

impl HtmlTag {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "div" => HtmlTag::Div,
            "p" => HtmlTag::P,
            "h1" => HtmlTag::H1,
            "h2" => HtmlTag::H2,
            "h3" => HtmlTag::H3,
            "body" => HtmlTag::Body,
            "html" => HtmlTag::Html,
            "head" => HtmlTag::Head,
            "span" => HtmlTag::Span,
            "a" => HtmlTag::A,
            "b" => HtmlTag::B,
            "strong" => HtmlTag::Strong,
            "img" => HtmlTag::Img,
            _ => HtmlTag::Other(s.to_string()),
        }
    }

    pub fn default_display(&self) -> DisplayType {
        match self {
            HtmlTag::Div
            | HtmlTag::P
            | HtmlTag::H1
            | HtmlTag::H2
            | HtmlTag::H3
            | HtmlTag::Body
            | HtmlTag::Html => DisplayType::Block,
            HtmlTag::Head => DisplayType::None,
            _ => DisplayType::Inline,
        }
    }

    // browser introduces default margins for certain tags.
    pub fn default_margin_px(&self) -> f32 {
        match self {
            HtmlTag::H1 => 21.0,
            HtmlTag::H2 => 19.0,
            HtmlTag::P => 16.0,
            HtmlTag::Body => 8.0,
            _ => 0.0,
        }
    }
}
