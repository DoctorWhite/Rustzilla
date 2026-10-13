use crate::dom::DomNode;
use crate::gui::font_set::FontSet;
use crate::layout::{DisplayableText, LayoutEngine};
use crate::style::{DisplayType, Unit};
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_filled_rect_mut;
use imageproc::rect::Rect;
use taffy::prelude::{Layout, NodeId};

use imageproc::drawing::draw_text_mut;
use rusttype::Scale;

pub struct Painter<'a> {
    canvas: RgbImage,
    fonts: &'a FontSet,
}

impl<'a> Painter<'a> {
    pub fn new(width: u32, height: u32, fonts: &'a FontSet) -> Self {
        let canvas = RgbImage::from_pixel(width, height, Rgb([255, 255, 255]));
        Self { canvas, fonts }
    }

    pub fn paint(&mut self, layout_engine: &LayoutEngine, dom_root: &DomNode) {
        self.paint_recursive(layout_engine, layout_engine.root_id, dom_root, 0.0, 0.0);
    }

    fn paint_recursive(
        &mut self,
        engine: &LayoutEngine,
        layout_id: NodeId,
        dom_node: &DomNode,
        parent_x: f32,
        parent_y: f32,
    ) {
        let layout = match engine.taffy.layout(layout_id) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to get layout: {}", e);
                return;
            }
        };

        let absolute_x = parent_x + layout.location.x;
        let absolute_y = parent_y + layout.location.y;

        let bg = dom_node.styles.background_color;
        if bg.a > 0 {
            let w = layout.size.width.max(1.0) as u32;
            let h = layout.size.height.max(1.0) as u32;
            let rect = Rect::at(absolute_x as i32, absolute_y as i32).of_size(w, h);
            let color_pixel = Rgb([bg.r, bg.g, bg.b]);
            draw_filled_rect_mut(&mut self.canvas, rect, color_pixel);
        }

        if let Some(text) = &dom_node.text {
            self.paint_text(dom_node, layout, absolute_x, absolute_y, text);
        }

        let taffy_children = match engine.taffy.children(layout_id) {
            Ok(children) => children,
            Err(_) => return,
        };

        let dom_children: Vec<&DomNode> = dom_node
            .children
            .iter()
            .filter(|c| c.styles.display != DisplayType::None)
            .collect();

        for (child_layout_id, child_dom) in taffy_children.iter().zip(dom_children.iter()) {
            self.paint_recursive(engine, *child_layout_id, child_dom, absolute_x, absolute_y);
        }
    }

    fn paint_text(
        &mut self,
        dom_node: &DomNode,
        layout: &Layout,
        absolute_x: f32,
        absolute_y: f32,
        text: &DisplayableText,
    ) {
        let pad_left = self.unit_to_px(dom_node.styles.padding_left, 0.0);
        let pad_top = self.unit_to_px(dom_node.styles.padding_top, 0.0);

        let mut text_start_x = absolute_x + pad_left;
        let text_start_y = absolute_y + pad_top;

        let font_size = dom_node.get_font_size();
        let active_font = self.fonts.get(dom_node.styles.font_weight);

        let color = dom_node.styles.color;
        let scale = Scale::uniform(font_size);
        let text_color = Rgb([color.r, color.g, color.b]);

        // feel free to simplify idk anymore
        if dom_node.styles.text_align != "left" {
            let str = match text {
                DisplayableText::Wrapped(w) => &*w.lines.join(" "),
                DisplayableText::Simple(text) => text,
            };
            let text_width = LayoutEngine::measure_text_naive(str, scale, active_font);

            let pad_right = self.unit_to_px(dom_node.styles.padding_right, 0.0);

            let content_box_width = layout.size.width - pad_left - pad_right;

            match dom_node.styles.text_align.as_str() {
                "center" => {
                    let offset = (content_box_width - text_width) / 2.0;
                    if offset > 0.0 {
                        text_start_x += offset;
                    }
                }
                "right" => {
                    let offset = content_box_width - text_width;
                    if offset > 0.0 {
                        text_start_x += offset;
                    }
                }
                _ => {}
            }
        }

        let v_metrics = active_font.v_metrics(scale);
        let glyph_height = v_metrics.ascent - v_metrics.descent;

        let line_gap = if v_metrics.line_gap > 0.0 {
            v_metrics.line_gap
        } else {
            font_size * 0.2
        };
        let line_height = glyph_height + line_gap;

        let mut y_cursor = text_start_y;
        match text {
            DisplayableText::Wrapped(wrapped) => {
                for line in &wrapped.lines {
                    draw_text_mut(
                        &mut self.canvas,
                        text_color,
                        text_start_x as i32,
                        y_cursor as i32,
                        scale,
                        active_font,
                        line,
                    );

                    y_cursor += line_height;
                }
            }

            DisplayableText::Simple(text) => {
                draw_text_mut(
                    &mut self.canvas,
                    text_color,
                    text_start_x as i32,
                    text_start_y as i32,
                    scale,
                    active_font,
                    text,
                );
            }
        }
    }

    pub fn save(&self, filename: &str) {
        self.canvas.save(filename).expect("Failed to save image");
    }

    fn unit_to_px(&self, unit: Unit, _parent_size: f32) -> f32 {
        match unit {
            Unit::Px(val) => val,
            _ => 0.0,
        }
    }

    pub fn as_rgba_bytes(&self) -> (Vec<u8>, usize, usize) {
        let (width, height) = self.canvas.dimensions();
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);

        for pixel in self.canvas.pixels() {
            rgba.push(pixel[0]); // R
            rgba.push(pixel[1]); // G
            rgba.push(pixel[2]); // B
            rgba.push(255); // A
        }

        (rgba, width as usize, height as usize)
    }
}
