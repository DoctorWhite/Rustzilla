use eframe::egui;
use std::path::Path;
use taffy::TaffyTree;
use taffy::prelude::NodeId;

use crate::gui::Renderer;
use crate::gui::renderer::RendererResources;
use crate::layout::LayoutEngine;

pub struct BrowserApp {
    resources: Option<RendererResources>,
    image: Option<egui::ColorImage>,
    texture: Option<egui::TextureHandle>,
    width: usize,

    layout_tree: Option<TaffyTree>,
    root_id: Option<NodeId>,

    current_dir: String,
}

impl BrowserApp {
    pub fn new() -> Self {
        Self {
            resources: None,
            image: None,
            texture: None,
            width: 0,
            layout_tree: None,
            root_id: None,
            current_dir: "examples/showcase_page".to_string(),
        }
    }

    fn render_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.image.is_none() {
                return;
            }

            if self.texture.is_none()
                && let Some(image) = &self.image
            {
                self.texture = Some(ui.ctx().load_texture(
                    "html-canvas",
                    image.clone(),
                    egui::TextureOptions::NEAREST,
                ));
            }
            let texture = self
                .texture
                .as_ref()
                .expect("Texture should exist - we just created it or checked it exists above")
                .clone();

            let response = egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add(egui::Image::new(&texture).sense(egui::Sense::click()))
                })
                .inner;

            if response.clicked() {
                self.handle_click(response);
            }
        });
    }

    fn handle_click(&mut self, response: egui::Response) {
        let Some(pointer_pos) = response.interact_pointer_pos() else {
            return;
        };

        let rel_x = pointer_pos.x - response.rect.min.x;
        let rel_y = pointer_pos.y - response.rect.min.y;

        if let Some(href) = self.perform_hit_test(rel_x, rel_y) {
            if cfg!(debug_assertions) {
                println!("Clicked on a link: {}", href);
            }
            self.navigate_to(&href);
        }
    }

    fn perform_hit_test(&self, x: f32, y: f32) -> Option<String> {
        let tree = self.layout_tree.as_ref()?;
        let root = self.root_id?;
        let res = self.resources.as_ref()?;
        LayoutEngine::hit_test(tree, root, &res.dom_root, x, y)
    }

    fn navigate_to(&mut self, href: &str) {
        if !href.ends_with(".html") {
            if cfg!(debug_assertions) {
                println!("Link is not HTML file, ignoring: {}", href);
            }
            return;
        }

        let current_path = Path::new(&self.current_dir);
        let new_full_path = current_path.join(href);

        match std::fs::canonicalize(&new_full_path) {
            Ok(absolute_path) => {
                if let Some(parent_dir) = absolute_path.parent() {
                    let new_dir = parent_dir.to_string_lossy().to_string();
                    let new_file = absolute_path.to_string_lossy().to_string();

                    self.load_page(&new_dir, &new_file);
                }
            }
            Err(e) => {
                eprintln!("Navigation error on {:?}: {}", new_full_path, e);
            }
        }
    }

    fn load_page(&mut self, dir: &str, file_path: &str) {
        if cfg!(debug_assertions) {
            println!("Loading page: {}", file_path);
        }

        if let Some(texture) = self.texture.take() {
            drop(texture);
        }

        self.current_dir = dir.to_string();

        let dir_path = Path::new(dir);

        let font_path = dir_path.join("times.ttf");
        let font_bold_path = dir_path.join("times-bold.ttf");

        self.resources = Some(Renderer::load_resources(
            dir,
            file_path,
            &font_path.to_string_lossy(),
            Some(&font_bold_path.to_string_lossy()),
        ));

        self.image = None;
        self.texture = None;
        self.layout_tree = None;
        self.root_id = None;
    }

    fn render_if_needed(&mut self, ctx: &egui::Context) {
        if self.resources.is_none() {
            let start_dir = "examples/showcase_page";
            self.load_page(start_dir, &format!("{}/index.html", start_dir));
        }

        let available = ctx.available_rect().size();
        let new_width = available.x.max(1.0) as usize;

        if (new_width != self.width || self.image.is_none())
            && let Some(resources) = self.resources.as_mut()
        {
            let renderer = Renderer::new(new_width);
            let result = renderer.render_layout(resources);

            self.width = result.width;
            self.image = Some(result.image);
            self.layout_tree = Some(result.taffy);
            self.root_id = Some(result.root_id);
            self.texture = None;
        }
    }
}

impl eframe::App for BrowserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_if_needed(ctx);
        self.render_ui(ctx);
    }
}

pub fn load_icon(path: &str) -> egui::IconData {
    let image = image::open(path)
        .unwrap_or_else(|e| panic!("Failed to load icon at '{}': {}", path, e))
        .into_rgba8();
    let (width, height) = image.dimensions();
    egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}
