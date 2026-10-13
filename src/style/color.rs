use css_color_parser::Color as ParsedColor;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    // Default colors for fallback
    pub fn black() -> Self {
        Color::new(0, 0, 0, 255)
    }
    pub fn transparent() -> Self {
        Color::new(0, 0, 0, 0)
    }

    pub fn from_string(s: &str) -> Self {
        match s.parse::<ParsedColor>() {
            Ok(c) => Color {
                r: c.r,
                g: c.g,
                b: c.b,
                a: (c.a * 255.0).round() as u8,
            },
            Err(_) => {
                println!("Warning: Unknown color '{}', using default.", s);
                Self::black()
            }
        }
    }
}
