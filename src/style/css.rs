use std::collections::HashMap;

use crate::dom::html_parser::ParsedElement;

const INHERITED_PROPS: &[&str] = &[
    "color",
    "font",
    "font-family",
    "font-size",
    "font-style",
    "font-weight",
    "font-variant",
    "line-height",
    "letter-spacing",
    "word-spacing",
    "text-align",
    "text-transform",
    "text-indent",
    "white-space",
    "direction",
    "visibility",
    "cursor",
];

#[derive(Debug, Clone)]
pub struct SimpleSelector {
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub universal: bool,
}

impl SimpleSelector {
    pub fn parse(input: &str) -> Self {
        let s = input.trim();
        if s.is_empty() {
            return Self {
                tag: None,
                id: None,
                classes: vec![],
                universal: false,
            };
        }

        let mut i = 0usize;
        let chars: Vec<char> = s.chars().collect();

        let mut universal = false;
        if chars.first() == Some(&'*') {
            universal = true;
            i += 1;
        }

        let mut tag = String::new();
        while i < chars.len() && chars[i] != '.' && chars[i] != '#' {
            tag.push(chars[i]);
            i += 1;
        }
        let tag = tag.trim();
        let tag_opt = if universal || tag.is_empty() {
            None
        } else {
            Some(tag.to_lowercase())
        };

        let mut id: Option<String> = None;
        let mut classes: Vec<String> = Vec::new();

        while i < chars.len() {
            match chars[i] {
                '.' => {
                    i += 1;
                    let mut name = String::new();
                    while i < chars.len() && chars[i] != '.' && chars[i] != '#' {
                        name.push(chars[i]);
                        i += 1;
                    }
                    let name = name.trim();
                    if !name.is_empty() {
                        classes.push(name.to_string());
                    }
                }
                '#' => {
                    i += 1;
                    let mut name = String::new();
                    while i < chars.len() && chars[i] != '.' && chars[i] != '#' {
                        name.push(chars[i]);
                        i += 1;
                    }
                    let name = name.trim();
                    if !name.is_empty() {
                        id = Some(name.to_string());
                    }
                }
                _ => i += 1,
            }
        }

        Self {
            tag: tag_opt,
            id,
            classes,
            universal,
        }
    }

    pub fn specificity(&self) -> u32 {
        let mut s = 0u32;
        if self.id.is_some() {
            s += 100;
        }
        s += (self.classes.len() as u32) * 10;
        if self.tag.is_some() {
            s += 1;
        }
        s
    }

    pub fn matches(&self, el: &ParsedElement) -> bool {
        if self.universal && self.tag.is_none() && self.id.is_none() && self.classes.is_empty() {
            return true;
        }

        if let Some(tag) = &self.tag
            && el.tag.to_lowercase() != *tag
        {
            return false;
        }

        if let Some(sel_id) = &self.id {
            match el.attributes.get("id") {
                Some(actual) if actual == sel_id => {}
                _ => return false,
            }
        }

        if !self.classes.is_empty() {
            let el_classes: Vec<&str> = el
                .attributes
                .get("class")
                .map(|s| s.split_whitespace().collect())
                .unwrap_or_default();

            for c in &self.classes {
                if !el_classes.contains(&c.as_str()) {
                    return false;
                }
            }
        }

        true
    }
}

#[derive(Debug, Clone)]
pub struct CssRule {
    pub selectors: Vec<SimpleSelector>,
    pub declarations: HashMap<String, String>,
    pub order: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
    pub rules: Vec<CssRule>,
}

impl Stylesheet {
    pub fn empty() -> Self {
        Self { rules: vec![] }
    }

    pub fn add_source(&mut self, css_text: &str) {
        let mut rules = parse_css(css_text, self.rules.len());
        self.rules.append(&mut rules);
    }

    pub fn from_sources(sources: &[String]) -> Self {
        let mut sheet = Stylesheet::empty();
        for src in sources {
            sheet.add_source(src);
        }
        sheet
    }

    pub fn specified_for(&self, el: &ParsedElement) -> HashMap<String, String> {
        let mut applied: HashMap<String, (u32, usize, String)> = HashMap::new();

        for rule in &self.rules {
            for sel in &rule.selectors {
                if !sel.matches(el) {
                    continue;
                }
                let spec = sel.specificity();

                for (prop, val) in &rule.declarations {
                    match applied.get(prop) {
                        None => {
                            applied.insert(prop.clone(), (spec, rule.order, val.clone()));
                        }
                        Some((cur_spec, cur_order, _)) => {
                            if spec > *cur_spec || (spec == *cur_spec && rule.order >= *cur_order) {
                                applied.insert(prop.clone(), (spec, rule.order, val.clone()));
                            }
                        }
                    }
                }
            }
        }

        applied.into_iter().map(|(k, (_, _, v))| (k, v)).collect()
    }

    pub fn compute_for(
        &self,
        el: &ParsedElement,
        parent: Option<&HashMap<String, String>>,
    ) -> HashMap<String, String> {
        let mut style = self.specified_for(el);
        match parent {
            Some(p) => {
                let inherit_keys: Vec<String> = style
                    .iter()
                    .filter(|(_, v)| *v == "inherit")
                    .map(|(k, _)| k.clone())
                    .collect();

                for k in inherit_keys {
                    inherit_or_remove(&mut style, p, k);
                }

                let unset_keys: Vec<String> = style
                    .iter()
                    .filter(|(_, v)| *v == "unset")
                    .map(|(k, _)| k.clone())
                    .collect();

                for k in unset_keys {
                    if INHERITED_PROPS.contains(&k.as_str()) {
                        inherit_or_remove(&mut style, p, k);
                    } else {
                        style.remove(&k);
                    }
                }
            }
            None => {
                let keys: Vec<String> = style
                    .iter()
                    .filter(|(_, v)| *v == "inherit" || *v == "unset")
                    .map(|(k, _)| k.clone())
                    .collect();
                for k in keys {
                    style.remove(&k);
                }
            }
        }
        let initial_keys: Vec<String> = style
            .iter()
            .filter(|(_, v)| *v == "initial")
            .map(|(k, _)| k.clone())
            .collect();
        for k in initial_keys {
            style.remove(&k);
        }

        if let Some(p) = parent {
            for prop in INHERITED_PROPS {
                if !style.contains_key(*prop)
                    && let Some(v) = p.get(*prop)
                {
                    style.insert((*prop).to_string(), v.clone());
                }
            }
        }

        style
    }
}

fn parse_css(css: &str, order_start: usize) -> Vec<CssRule> {
    let css = strip_comments(css);

    let mut rules: Vec<CssRule> = Vec::new();
    let mut order = order_start;

    let mut rest = css.as_str();
    while let Some(open) = rest.find('{') {
        let (sel_part, after_sel) = rest.split_at(open);
        let after_sel = &after_sel[1..]; // skip '{'

        let Some(close) = after_sel.find('}') else {
            break;
        };

        let (decl_part, after_decl) = after_sel.split_at(close);
        rest = &after_decl[1..]; // skip '}'

        let selectors_raw = sel_part.trim();
        if selectors_raw.is_empty() {
            continue;
        }

        let selectors: Vec<SimpleSelector> = selectors_raw
            .split(',')
            .map(SimpleSelector::parse)
            .collect();

        let declarations = parse_declarations(decl_part);

        rules.push(CssRule {
            selectors,
            declarations,
            order,
        });
        order += 1;
    }

    rules
}

fn parse_declarations(decls: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for declaration in decls.split(';') {
        let d = declaration.trim();
        if d.is_empty() {
            continue;
        }
        if let Some((prop, val)) = d.split_once(':') {
            map.insert(prop.trim().to_string(), val.trim().to_string());
        }
    }

    map
}

fn strip_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut chars = css.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            while let Some(x) = chars.next() {
                if x == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }

    out
}

fn inherit_or_remove(
    style: &mut HashMap<String, String>,
    parent: &HashMap<String, String>,
    key: String,
) {
    if let Some(pv) = parent.get(&key) {
        style.insert(key, pv.clone());
    } else {
        style.remove(&key);
    }
}
