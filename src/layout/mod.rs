use crate::dom::DomNode;
use crate::style::{DisplayType, Unit};
use taffy::prelude::*;

use crate::gui::font_set::FontSet;
use taffy::TaffyError;
use taffy::TaffyTree;

use crate::layout::DisplayableText::{Simple, Wrapped};
use rusttype::{Font, Scale};

#[derive(Debug, Clone)]
pub struct WrappedText {
    pub lines: Vec<String>,
    pub width: f32,
    pub height: f32,
}

impl WrappedText {
    pub fn from_string(str: &str, w: f32, h: f32) -> WrappedText {
        WrappedText {
            lines: str.lines().map(|s| s.to_string()).collect(),
            width: w,
            height: h,
        }
    }

    pub fn to_style(&self, ref_style: Style) -> Style {
        Style {
            size: Size {
                width: Dimension::Length(self.width),
                height: Dimension::Length(self.height),
            },
            ..ref_style
        }
    }
}

#[derive(Debug, Clone)]
pub enum DisplayableText {
    Wrapped(WrappedText),
    Simple(String),
}

pub struct LayoutEngine<'a> {
    pub taffy: TaffyTree,
    pub root_id: NodeId,
    pub fonts: &'a FontSet,
}

impl<'a> LayoutEngine<'a> {
    pub fn new(fonts: &'a FontSet) -> Self {
        let mut taffy = TaffyTree::new();
        let root_id = taffy
            .new_leaf(Style::default())
            .expect("Failed to create root layout node - Taffy tree initialization failed");

        Self {
            taffy,
            root_id,
            fonts,
        }
    }

    pub fn calculate_layout(
        &mut self,
        dom_root: &mut DomNode,
        width: f32,
        height: f32,
    ) -> Result<(), TaffyError> {
        self.taffy = TaffyTree::new();

        self.root_id = self.build_tree(dom_root, None)?;

        let window_size = Size {
            width: AvailableSpace::Definite(width),
            height: AvailableSpace::Definite(height),
        };
        self.taffy.compute_layout(self.root_id, window_size)?;

        let layout = self.taffy.layout(self.root_id)?;
        let computed_width = layout.size.width;
        self.root_id = self.build_tree(dom_root, Some(computed_width))?;
        self.taffy.compute_layout(self.root_id, window_size)?;

        Ok(())
    }

    fn build_tree(
        &mut self,
        node: &mut DomNode,
        max_width: Option<f32>,
    ) -> Result<NodeId, TaffyError> {
        let old_style = self.convert_style(&node.styles);

        if node.text.is_none() {
            let children_ids = self.collect_children(node, max_width)?;
            return self.taffy.new_with_children(old_style, &children_ids);
        }

        let font = self.fonts.get(node.styles.font_weight);

        let font_size = node.get_font_size();
        let scale = Scale::uniform(font_size);

        let text_node_style = Style::default();
        let (text_node_id, new_text) = match max_width {
            None => {
                let text = node.text.as_ref().expect("node.text should be Some here");
                match text {
                    Simple(str) => (
                        self.taffy.new_leaf(text_node_style)?,
                        Simple(str.to_string()),
                    ),
                    Wrapped(t) => (
                        self.taffy.new_leaf(text_node_style)?,
                        Simple(t.lines.join(" ")),
                    ),
                }
            }

            Some(max_width) => {
                let text = node.text.take().expect("node.text should be Some here");
                match text {
                    Wrapped(t) => (
                        self.taffy.new_leaf(t.to_style(text_node_style))?,
                        Wrapped(t),
                    ),

                    Simple(str) => {
                        let v_metrics = font.v_metrics(scale);
                        let glyph_height = v_metrics.ascent - v_metrics.descent;

                        let line_gap = if v_metrics.line_gap > 0.0 {
                            v_metrics.line_gap
                        } else {
                            font_size * 0.2
                        };
                        let line_height = glyph_height + line_gap;
                        let wrapped = Self::wrap_text(&str, scale, max_width, font, line_height);
                        let text_style = wrapped.to_style(text_node_style);
                        (self.taffy.new_leaf(text_style)?, Wrapped(wrapped))
                    }
                }
            }
        };
        node.text = Some(new_text);

        // separate text from its container
        self.taffy.new_with_children(old_style, &[text_node_id; 1])
    }

    fn collect_children(
        &mut self,
        node: &mut DomNode,
        max_width: Option<f32>,
    ) -> Result<Vec<NodeId>, TaffyError> {
        let mut children_ids = Vec::new();

        for child in &mut node.children {
            let fun = |margin| match margin {
                Unit::Px(px) => px,
                Unit::Pct(pct) => (pct / 100.) * max_width.unwrap_or(0.),
                Unit::Auto => max_width.unwrap_or(0.),
                Unit::Zero => 0.,
            };
            if child.styles.display != DisplayType::None {
                let margin_l: f32 = fun(child.styles.margin_left);
                let margin_r: f32 = fun(child.styles.margin_right);

                let child_id =
                    self.build_tree(child, max_width.map(|w: f32| w - (margin_l + margin_r)))?;

                children_ids.push(child_id);
            }
        }
        Ok(children_ids)
    }

    fn convert_style(&self, my_style: &crate::style::StyleMap) -> Style {
        Style {
            display: match my_style.display {
                DisplayType::None => Display::None,
                DisplayType::Block => Display::Block,
                DisplayType::Inline => Display::Flex,
                DisplayType::Flex => Display::Flex,
            },

            box_sizing: BoxSizing::ContentBox,

            size: Size {
                width: self.to_dimension(my_style.width),
                height: self.to_dimension(my_style.height),
            },

            margin: Rect {
                left: self.to_length_auto(my_style.margin_left),
                right: self.to_length_auto(my_style.margin_right),
                top: self.to_length_auto(my_style.margin_top),
                bottom: self.to_length_auto(my_style.margin_bottom),
            },

            padding: Rect {
                left: self.to_length_percentage(my_style.padding_left),
                right: self.to_length_percentage(my_style.padding_right),
                top: self.to_length_percentage(my_style.padding_top),
                bottom: self.to_length_percentage(my_style.padding_bottom),
            },

            flex_direction: match my_style.flex_direction.as_str() {
                "row" => FlexDirection::Row,
                "column-reverse" => FlexDirection::ColumnReverse,
                _ => FlexDirection::Column,
            },

            align_items: Some(AlignItems::FlexStart),

            ..Default::default()
        }
    }

    fn to_dimension(&self, unit: Unit) -> Dimension {
        match unit {
            Unit::Auto => Dimension::Auto,
            Unit::Px(v) => Dimension::Length(v),
            Unit::Pct(v) => Dimension::Percent(v / 100.0),
            Unit::Zero => Dimension::Length(0.0),
        }
    }

    fn to_length_percentage(&self, unit: Unit) -> LengthPercentage {
        match unit {
            Unit::Px(v) => LengthPercentage::Length(v),
            Unit::Pct(v) => LengthPercentage::Percent(v / 100.0),
            Unit::Zero => LengthPercentage::Length(0.0),
            Unit::Auto => LengthPercentage::Length(0.0),
        }
    }

    fn to_length_auto(&self, unit: Unit) -> LengthPercentageAuto {
        match unit {
            Unit::Auto => LengthPercentageAuto::Auto,
            Unit::Px(v) => LengthPercentageAuto::Length(v),
            Unit::Pct(v) => LengthPercentageAuto::Percent(v / 100.0),
            Unit::Zero => LengthPercentageAuto::Length(0.0),
        }
    }

    pub fn measure_text_naive(text: &str, scale: Scale, font: &Font) -> f32 {
        font.layout(text, scale, rusttype::point(0.0, 0.0))
            .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
            .last()
            .unwrap_or(0.0)
    }

    fn wrap_text(
        text: &str,
        scale: Scale,
        max_width: f32,
        font: &Font,
        line_height: f32,
    ) -> WrappedText {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0.0;

        let space_width = Self::measure_text_naive(" ", scale, font);
        for word in text.split_whitespace() {
            let word_width = Self::measure_text_naive(word, scale, font);

            // space width for simplicity
            if current_width + word_width + space_width > max_width && !current_line.is_empty() {
                lines.push(current_line);
                current_line = String::new();
                current_width = 0.0;
            }

            if !current_line.is_empty() {
                current_line.push(' ');
                current_width += space_width;
            }

            current_line.push_str(word);
            current_width += word_width;
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        WrappedText {
            width: current_width,
            height: line_height * lines.len() as f32,
            lines,
        }
    }

    pub fn print_layout(&self) {
        self.print_layout_recursive(self.root_id, 0);
    }
    pub fn print_layout_recursive(&self, node_id: NodeId, depth: usize) {
        let layout = self
            .taffy
            .layout(node_id)
            .expect("Failed to get layout for node during painting - layout tree is corrupted");

        let indent = "  ".repeat(depth);

        println!(
            "{}Box: x={:.1}, y={:.1}, w={:.1}, h={:.1}",
            indent, layout.location.x, layout.location.y, layout.size.width, layout.size.height
        );

        let children = self
            .taffy
            .children(node_id)
            .expect("Failed to get children for node during painting - layout tree is corrupted");

        for child in children {
            self.print_layout_recursive(child, depth + 1);
        }
    }

    pub fn hit_test(
        taffy: &TaffyTree,
        root_id: NodeId,
        dom_node: &DomNode,
        x: f32,
        y: f32,
    ) -> Option<String> {
        Self::hit_test_recursive(taffy, root_id, dom_node, 0.0, 0.0, x, y)
    }
    fn hit_test_recursive(
        taffy: &TaffyTree,
        layout_id: NodeId,
        dom_node: &DomNode,
        parent_x: f32,
        parent_y: f32,
        click_x: f32,
        click_y: f32,
    ) -> Option<String> {
        let layout = taffy.layout(layout_id).ok()?;

        let absolute_x = parent_x + layout.location.x;
        let absolute_y = parent_y + layout.location.y;
        let width = layout.size.width;
        let height = layout.size.height;

        let is_hit = click_x >= absolute_x
            && click_x <= absolute_x + width
            && click_y >= absolute_y
            && click_y <= absolute_y + height;

        if !is_hit {
            return None;
        }

        let taffy_children = taffy.children(layout_id).unwrap_or_default();

        let visible_dom_children: Vec<&DomNode> = dom_node
            .children
            .iter()
            .filter(|c| !c.is_display_none())
            .collect();

        for (i, child_dom) in visible_dom_children.iter().enumerate() {
            if i < taffy_children.len()
                && let Some(url) = Self::hit_test_recursive(
                    taffy,
                    taffy_children[i],
                    child_dom,
                    absolute_x,
                    absolute_y,
                    click_x,
                    click_y,
                )
            {
                return Some(url);
            }
        }

        if let Some(href) = &dom_node.href {
            return Some(href.clone());
        }

        None
    }
}
