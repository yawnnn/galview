mod app;
mod common;
mod ui;
mod logging;

use crate::app::*;

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };
    eframe::run_native(
        "Galview",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            disable_animations(&cc.egui_ctx);
            Ok(Box::new(App::new()))
        }),
    )
}
