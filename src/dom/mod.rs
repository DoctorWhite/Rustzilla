pub mod html_parser;
pub mod tag;

use crate::style::css::Stylesheet;
use crate::style::{Color, DisplayType, StyleMap, Unit};
use html_parser::ParsedElement;

use crate::layout::DisplayableText;
use crate::layout::DisplayableText::Simple;
use std::collections::HashMap;
use tag::HtmlTag;

#[derive(Debug, Clone)]
pub struct DomNode {
    pub tag: HtmlTag,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub styles: StyleMap,
    pub children: Vec<DomNode>,
    pub text: Option<DisplayableText>,
    pub href: Option<String>,
}

impl DomNode {
    pub fn get_font_size(&self) -> f32 {
        match self.styles.font_size {
            Unit::Px(val) => val,
            _ => 16.0,
        }
    }

    pub fn is_display_none(&self) -> bool {
        self.styles.display == DisplayType::None
    }

    pub fn print(&self) {
        self.print_dom_recursive(0);
    }

    pub fn print_dom_recursive(&self, depth: usize) {
        let indent = "  ".repeat(depth);

        println!("{}Node: <{:?}>", indent, self.tag);

        if let Some(id) = &self.id {
            println!("{}  ID: #{}", indent, id);
        }

        if !self.classes.is_empty() {
            let class_str = self.classes.join(".");
            println!("{}  Classes: .{}", indent, class_str);
        }

        println!("{}  Computed Styles:", indent);
        println!("{}    Display: {:?}", indent, self.styles.display);
        println!("{}    Width:   {:?}", indent, self.styles.width);
        println!("{}    Height:  {:?}", indent, self.styles.height);
        println!(
            "{}    Margin:  top: {:?}, bottom: {:?}, left: {:?}, right: {:?}",
            indent,
            self.styles.margin_top,
            self.styles.margin_bottom,
            self.styles.margin_left,
            self.styles.margin_right
        );
        println!("{}    Color:   {:?}", indent, self.styles.color);
        println!(
            "{}    Background Color: {:?}",
            indent, self.styles.background_color
        );

        if let Some(text) = &self.text {
            println!("{}  Text: {:?}", indent, text);
        }

        for child in &self.children {
            child.print_dom_recursive(depth + 1);
        }
    }
}

pub struct NodeConverter;

impl NodeConverter {
    pub fn convert(parsed: ParsedElement) -> DomNode {
        Self::convert_with_stylesheet(parsed, &Stylesheet::empty())
    }

    pub fn convert_with_stylesheet(parsed: ParsedElement, stylesheet: &Stylesheet) -> DomNode {
        Self::convert_with_stylesheet_inner(parsed, stylesheet, None)
    }

    fn convert_with_stylesheet_inner(
        parsed: ParsedElement,
        stylesheet: &Stylesheet,
        parent_css: Option<&HashMap<String, String>>,
    ) -> DomNode {
        let tag = HtmlTag::from_string(&parsed.tag);
        let id = parsed.attributes.get("id").cloned();
        let classes = parsed
            .attributes
            .get("class")
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default();

        let href = parsed.attributes.get("href").cloned();

        let mut combined: HashMap<String, String> = stylesheet.compute_for(&parsed, parent_css);

        let inline_styles = parsed.get_styles();
        for (k, v) in inline_styles {
            combined.insert(k, v);
        }

        let styles = Self::parse_styles(&combined, &tag);

        let children = parsed
            .children
            .into_iter()
            .filter(|child| {
                let tag = child.tag.to_lowercase();
                tag != "style" && tag != "script" && tag != "head"
            })
            .map(|c| Self::convert_with_stylesheet_inner(c, stylesheet, Some(&combined)))
            .collect();

        let text = parsed.text.map(Simple);

        DomNode {
            tag,
            id,
            classes,
            styles,
            children,
            text,
            href,
        }
    }

    fn parse_styles(css: &HashMap<String, String>, tag: &HtmlTag) -> StyleMap {
        let get_val = |key| css.get(key).map(|s| s.as_str());

        let display = if let Some(val) = get_val("display") {
            match val {
                "block" => DisplayType::Block,
                "inline" => DisplayType::Inline,
                "none" => DisplayType::None,
                "flex" => DisplayType::Flex,
                _ => tag.default_display(),
            }
        } else {
            tag.default_display()
        };

        let width = get_val("width").map(Self::parse_unit).unwrap_or(Unit::Auto);
        let height = get_val("height")
            .map(Self::parse_unit)
            .unwrap_or(Unit::Auto);

        let (mut mt, mut mb, mut ml, mut mr) = if let Some(m_str) = get_val("margin") {
            let u = Self::parse_unit(m_str);
            (u, u, u, u)
        } else {
            let def_vert = tag.default_margin_px();
            let u_vert = if def_vert > 0.0 {
                Unit::Px(def_vert)
            } else {
                Unit::Zero
            };
            (u_vert, u_vert, Unit::Zero, Unit::Zero)
        };

        if let Some(s) = get_val("margin-top") {
            mt = Self::parse_unit(s);
        }
        if let Some(s) = get_val("margin-bottom") {
            mb = Self::parse_unit(s);
        }
        if let Some(s) = get_val("margin-left") {
            ml = Self::parse_unit(s);
        }
        if let Some(s) = get_val("margin-right") {
            mr = Self::parse_unit(s);
        }

        let color = get_val("color").map(Color::from_string).unwrap_or_else(|| {
            match tag {
                // Blue color for links
                HtmlTag::A => Color::new(0, 0, 238, 255),
                _ => Color::black(),
            }
        });
        let background_color = get_val("background-color")
            .map(Color::from_string)
            .unwrap_or(Color::transparent());

        let default_size = match tag {
            HtmlTag::H1 => 32.0,
            HtmlTag::H2 => 24.0,
            HtmlTag::H3 => 18.72,
            _ => 16.0,
        };

        let font_size = get_val("font-size")
            .map(Self::parse_unit)
            .unwrap_or(Unit::Px(default_size));

        let font_weight = if let Some(w_str) = get_val("font-weight") {
            match w_str {
                "bold" | "700" | "800" | "900" => 700,
                "normal" | "400" => 400,
                _ => 400,
            }
        } else {
            match tag {
                HtmlTag::H1 | HtmlTag::H2 | HtmlTag::H3 => 700,
                _ => 400,
            }
        };

        let flex_direction = get_val("flex-direction").unwrap_or("column").to_string();

        let (mut pt, mut pb, mut pl, mut pr) = if let Some(p_str) = get_val("padding") {
            let u = Self::parse_unit(p_str);
            (u, u, u, u)
        } else {
            (Unit::Zero, Unit::Zero, Unit::Zero, Unit::Zero)
        };

        if let Some(s) = get_val("padding-top") {
            pt = Self::parse_unit(s);
        }
        if let Some(s) = get_val("padding-bottom") {
            pb = Self::parse_unit(s);
        }
        if let Some(s) = get_val("padding-left") {
            pl = Self::parse_unit(s);
        }
        if let Some(s) = get_val("padding-right") {
            pr = Self::parse_unit(s);
        }

        StyleMap {
            display,
            width,
            height,
            color,
            background_color,
            margin_top: mt,
            margin_bottom: mb,
            margin_left: ml,
            margin_right: mr,
            font_size,
            font_weight,
            text_align: get_val("text-align").unwrap_or("left").to_string(),
            flex_direction,
            padding_top: pt,
            padding_bottom: pb,
            padding_left: pl,
            padding_right: pr,
        }
    }

    fn parse_unit(value: &str) -> Unit {
        let v = value.trim();
        if v == "auto" {
            return Unit::Auto;
        }
        if v == "0" {
            return Unit::Zero;
        }

        if v.ends_with("px") {
            if let Ok(num) = v.trim_end_matches("px").parse::<f32>() {
                return Unit::Px(num);
            }
        } else if v.ends_with('%')
            && let Ok(num) = v.trim_end_matches('%').parse::<f32>()
        {
            return Unit::Pct(num);
        }
        Unit::Auto
    }
}
