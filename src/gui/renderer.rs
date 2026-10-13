use crate::dom::{DomNode, NodeConverter, html_parser::HtmlParser};
use crate::gui::{font_set::FontSet, painter::Painter};
use crate::layout::LayoutEngine;
use crate::style::css::Stylesheet;
use std::path::Path;

use egui::ColorImage;
use regex::Regex;
use std::fs;
use taffy::TaffyTree;
use taffy::prelude::NodeId;

pub struct RendererResources {
    pub dom_root: DomNode,
    pub fonts: FontSet,
}

pub struct RenderResult {
    pub image: ColorImage,
    pub width: usize,
    pub taffy: TaffyTree,
    pub root_id: NodeId,
}

pub struct Renderer {
    pub width: usize,
}

impl Renderer {
    pub fn new(width: usize) -> Self {
        Self { width }
    }

    pub fn load_resources(
        dir_path: &str,
        html_path: &str,
        font_path: &str,
        font_bold_path: Option<&str>,
    ) -> RendererResources {
        let base_dir = Path::new(dir_path);

        let html = fs::read_to_string(html_path).expect("Failed to read HTML");
        // Normalize whitespace
        let re = Regex::new(r"\s+").unwrap();
        let html = re.replace_all(&html, " ").to_string();

        let parsed = HtmlParser::parse(&html);
        let (links, inline) = HtmlParser::collect_css_sources(&parsed);

        let mut css_sources: Vec<String> = links
            .clone()
            .into_iter()
            .filter_map(|href| fs::read_to_string(href).ok())
            .collect();
        css_sources.extend(inline);
        for href in links {
            let css_path = base_dir.join(href);
            if let Ok(css) = fs::read_to_string(&css_path) {
                css_sources.push(css);
            }
        }

        let stylesheet = Stylesheet::from_sources(&css_sources);
        let dom_root = NodeConverter::convert_with_stylesheet(parsed, &stylesheet);

        let reg_data = fs::read(font_path).expect("Failed to read regular font");
        let bold_data = font_bold_path
            .map(|p| fs::read(p).expect("Failed to read bold font"))
            .unwrap_or_else(|| reg_data.clone());

        RendererResources {
            dom_root,
            fonts: FontSet::new(reg_data, bold_data),
        }
    }

    pub fn render_layout(&self, resources: &mut RendererResources) -> RenderResult {
        // CREATE LAYOUT
        let mut layout_engine = LayoutEngine::new(&resources.fonts);
        layout_engine
            .calculate_layout(
                &mut resources.dom_root,
                (self.width - 15) as f32,
                f32::INFINITY,
            )
            .expect("Layout failed");

        let root_layout = layout_engine.taffy.layout(layout_engine.root_id).unwrap();

        let document_height = root_layout.size.height.ceil() as usize;

        if cfg!(debug_assertions) {
            layout_engine.print_layout();
        }

        // CREATE IMAGE
        let mut painter = Painter::new(
            self.width as u32,
            document_height.max(1) as u32,
            &resources.fonts,
        );
        painter.paint(&layout_engine, &resources.dom_root);

        if cfg!(debug_assertions) {
            painter.save("debug_output.png");
        }

        let (pixels, width, height) = painter.as_rgba_bytes();
        let image = ColorImage::from_rgba_unmultiplied([width, height], &pixels);

        let taffy = layout_engine.taffy;
        let root_id = layout_engine.root_id;

        RenderResult {
            image,
            width: self.width,
            taffy,
            root_id,
        }
    }
}
