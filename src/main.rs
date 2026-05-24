use std::{
    fs,
    ops::Rem,
    path::{Path, PathBuf},
};

use eframe::egui;
use egui::{AtomExt, CentralPanel, Grid, Image, ImageSource, ScrollArea, Vec2};

use crate::common::{new_frame, resource};

mod common;

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1920., 1080.])
            .with_clamp_size_to_monitor_size(true),
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

#[derive(Default)]
struct App {
    dir: PathBuf,
}

impl App {
    fn new() -> Self {
        let root = PathBuf::from(std::env::var("HOME").unwrap()).join("Pictures");
        App { dir: root }
    }

    fn curr_entries(&self) -> Vec<PathBuf> {
        self.dir
            .read_dir()
            .unwrap_or_else(|e| panic!("{:?}: {e}", self.dir))
            .filter_map(|e| e.ok())
            //.filter(|e| e.metadata().unwrap().is_file())
            .map(|e| e.path())
            .collect()
    }

    fn folder_find_thumbnail(folder: &Path) -> Option<PathBuf> {
        for e in folder.read_dir().ok()? {
            let e = e.ok()?;
            if e.metadata().unwrap().is_file() {
                return Some(e.path());
            }
        }

        for e in folder.read_dir().ok()? {
            let e = e.ok()?;
            if e.metadata().unwrap().is_dir()
                && let Some(p) = Self::folder_find_thumbnail(&e.path())
            {
                return Some(p);
            }
        }

        None
    }

    fn draw_thumbnail(&self, ui: &mut egui::Ui, label: &str, path: &Path, cell_size: Vec2) {
        let uri = format!("file://{}", path.to_str().unwrap());
        let img = Image::from_uri(uri).fit_to_exact_size(cell_size);
        ui.add(img).on_hover_text_at_pointer(label);
    }

    fn draw_thumbnails(&self, ui: &mut egui::Ui, entries: &[PathBuf]) {
        let spacing = ui.spacing().item_spacing; // [8, 3]
        let max_size = ui.available_size(); // 1904, 1064
        // let aspect_ratio = max_size.x / max_size.y;
        let max_rows = 5;
        let max_cols = 5;
        let max_cells = max_rows * max_cols;
        //let min_cell_size = (max_size + spacing) / max_grid; // (382.4, 213.4)
        let n_cells = entries.len().min(max_cells); // TODO: len == 0
        let n_rows = (n_cells as f32 / max_rows as f32).ceil() as usize; // prioritize laying out rows
        let n_cols = n_cells / n_rows;
        let rest = n_cells - (n_rows * n_cols);
        let n_rows = n_rows + if rest > 0 { 1 } else { 0 };
        let cell_size = (max_size + spacing) / Vec2::new(n_rows as f32, n_cols as f32);

        ui.columns(n_cols, |cols| {
            for (i, entry) in entries.iter().enumerate() {
                let col = &mut cols[i % n_cols];
                col.vertical_centered(|ui| {
                    let label = entry.file_name().unwrap().to_str().unwrap();
                    if entry.metadata().unwrap().is_file() {
                        self.draw_thumbnail(ui, label, entry, cell_size);
                    } else if let Some(p) = Self::folder_find_thumbnail(entry) {
                        self.draw_thumbnail(ui, label, &p, cell_size);
                    } else {
                        self.draw_thumbnail(ui, label, &resource("folder.svg"), cell_size);
                    }
                });
            }
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        new_frame();

        CentralPanel::default().show_inside(ui, |ui| {
            ScrollArea::both().show(ui, |ui| {
                self.draw_thumbnails(ui, &self.curr_entries());
            });
        });
    }
}
