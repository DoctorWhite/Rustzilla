use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ParsedElement {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<ParsedElement>,
    pub text: Option<String>,
}

pub struct HtmlParser;

impl HtmlParser {
    pub fn parse(html: &str) -> ParsedElement {
        let document = Html::parse_document(html);
        let root_selector = Selector::parse("html").expect("HTML file does not contain <html> tag");

        if let Some(root) = document.select(&root_selector).next() {
            Self::parse_element(root)
        } else {
            let body_selector =
                Selector::parse("body").expect("HTML file does not contain <body> tag");
            if let Some(body) = document.select(&body_selector).next() {
                Self::parse_element(body)
            } else {
                ParsedElement {
                    tag: "root".to_string(),
                    attributes: HashMap::new(),
                    children: vec![],
                    text: None,
                }
            }
        }
    }

    fn parse_element(element: ElementRef) -> ParsedElement {
        let tag = element.value().name().to_string();

        let mut attributes = HashMap::new();
        for attr in element.value().attrs() {
            attributes.insert(attr.0.to_string(), attr.1.to_string());
        }

        if tag.eq_ignore_ascii_case("style") {
            let raw = element.inner_html();
            let text = raw.trim();

            return ParsedElement {
                tag,
                attributes,
                children: vec![],
                text: if text.is_empty() {
                    None
                } else {
                    Some(text.to_string())
                },
            };
        }

        let mut children = Vec::new();
        let mut text_content = String::new();

        for child in element.children() {
            if let Some(child_element) = ElementRef::wrap(child) {
                children.push(Self::parse_element(child_element));
            } else if let Some(text) = child.value().as_text() {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    text_content.push_str(trimmed);
                }
            }
        }

        ParsedElement {
            tag,
            attributes,
            children,
            text: if text_content.is_empty() {
                None
            } else {
                Some(text_content)
            },
        }
    }

    pub fn parse_inline_styles(style_attr: &str) -> HashMap<String, String> {
        let mut styles = HashMap::new();

        for declaration in style_attr.split(';') {
            let declaration = declaration.trim();
            if let Some((property, value)) = declaration.split_once(':') {
                styles.insert(property.trim().to_string(), value.trim().to_string());
            }
        }

        styles
    }

    pub fn collect_css_sources(root: &ParsedElement) -> (Vec<String>, Vec<String>) {
        let mut links = Vec::new();
        let mut inline = Vec::new();
        Self::collect_css_sources_rec(root, &mut links, &mut inline);
        (links, inline)
    }

    fn collect_css_sources_rec(
        node: &ParsedElement,
        links: &mut Vec<String>,
        inline: &mut Vec<String>,
    ) {
        let tag = node.tag.to_lowercase();

        if tag == "link" {
            let rel_ok = node
                .attributes
                .get("rel")
                .map(|r| r.to_lowercase().contains("stylesheet"))
                .unwrap_or(false);

            if rel_ok && let Some(href) = node.attributes.get("href") {
                links.push(href.clone());
            }
        }

        if tag == "style"
            && let Some(text) = &node.text
            && !text.trim().is_empty()
        {
            inline.push(text.clone());
        }

        for ch in &node.children {
            Self::collect_css_sources_rec(ch, links, inline);
        }
    }
}

impl ParsedElement {
    pub fn get_styles(&self) -> HashMap<String, String> {
        if let Some(style_str) = self.attributes.get("style") {
            HtmlParser::parse_inline_styles(style_str)
        } else {
            HashMap::new()
        }
    }

    pub fn get_attr(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }

    pub fn debug_print(&self, indent: usize) {
        let indent_str = "  ".repeat(indent);
        println!("{}Tag: {}", indent_str, self.tag);

        if !self.attributes.is_empty() {
            println!("{}Attributes: {:?}", indent_str, self.attributes);
        }

        if let Some(text) = &self.text {
            println!("{}Text: {}", indent_str, text);
        }

        for child in &self.children {
            child.debug_print(indent + 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_html() {
        let html = r#"
            <html>
                <head>
                    <title>Franta</title>
                </head>
                <body style="background-color: #f0f0f0; margin: 0;">
                    <div style="color: red; font-size: 16px;">
                        <h1 style="font-weight: bold;">Hello World</h1>
                        <p>This is a paragraph</p>
                    </div>
                </body>
            </html>
        "#;

        let parsed = HtmlParser::parse(html);

        if let Some(body) = parsed.children.get(1) {
            let styles = body.get_styles();
            assert_eq!(styles.get("background-color"), Some(&"#f0f0f0".to_string()));
        }
    }

    #[test]
    fn test_parse_link_tag() {
        let html = r#"
            <body style="background-color: #f0f0f0; margin: 0;">
                <div style="color: red; font-size: 16px;">
                    <h1 style="font-weight: bold;">Hello World</h1>
                    <p>This is a paragraph</p>
                    <a href="bangladesh.blg">Bangladesh</a>
                </div>
            </body>
        "#;

        let parsed = HtmlParser::parse(html);

        if let Some(body) = parsed.children.first() {
            if let Some(div) = body.children.first() {
                if let Some(link) = div.children.get(2) {
                    let link_path = link.get_attr("href");
                    assert_eq!(link.tag, "a");
                    assert_eq!(link_path, Some(&"bangladesh.blg".to_string()));
                    assert_eq!(link.text, Some("Bangladesh".to_string()));
                }
            }
        }
    }
}
