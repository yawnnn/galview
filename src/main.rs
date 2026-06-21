mod app;
mod ui;
mod common;

use crate::app::App;

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
        .with_maximized(true),
        ..Default::default()
    };
    eframe::run_native(
        "Galview",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(App::new()))
        }),
    )
}