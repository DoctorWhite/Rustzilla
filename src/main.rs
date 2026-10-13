use eframe::egui;

mod gui;

pub mod dom;
pub mod layout;
pub mod style;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(gui::load_icon("logo.png"))
            .with_maximized(true),

        ..Default::default()
    };

    eframe::run_native(
        "RUSTZILLA",
        options,
        Box::new(|_cc| Ok(Box::new(gui::BrowserApp::new()))),
    )
}
