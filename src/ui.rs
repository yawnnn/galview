use eframe::egui;
use egui::{CentralPanel, Event, Image, Key, ScrollArea, Ui, Vec2};
use std::path::Path;

use crate::App;

pub trait UiExt {
    fn keys_pressed(&self) -> Vec<Key>;
}

impl UiExt for Ui {
    fn keys_pressed(&self) -> Vec<Key> {
        let mut keys = Vec::new();
        self.input(|i| {
            keys.extend(i.events.iter().filter_map(|e| match e {
                Event::Key {
                    key, pressed: true, ..
                } => Some(*key),
                _ => None,
            }));
        });
        keys
    }
}

impl App {
    fn draw_thumbnail(&self, ui: &mut Ui, label: &str, path: &Path, cell_size: Vec2, cursor: bool) {
        let uri = format!("file://{}", path.to_str().unwrap());
        let img = Image::from_uri(uri).fit_to_exact_size(cell_size);
        let resp = ui.add(img).on_hover_text_at_pointer(label);

        if cursor {
            ui.painter().rect_stroke(
                resp.rect,
                0.,
                egui::Stroke::new(3.0f32, egui::Color32::from_rgb(255, 200, 0)),
                egui::StrokeKind::Outside,
            );
        }
    }

    fn draw_thumbnails(&mut self, ui: &mut Ui) {
        let spacing = ui.spacing().item_spacing; // [8, 3]
        let max_size = ui.available_size(); // 1904, 1064
        // let aspect_ratio = max_size.x / max_size.y;
        let max_rows = 5;
        let max_cols = 5;
        let max_cells = max_rows * max_cols;
        //let min_cell_size = (max_size + spacing) / max_grid; // (382.4, 213.4)
        let n_cells = self.entries.len().min(max_cells); // TODO: len == 0
        let n_rows = (n_cells as f32 / max_rows as f32).ceil() as usize; // prioritize laying out rows
        let n_cols = n_cells / n_rows;
        let rest = n_cells - (n_rows * n_cols);
        let n_rows = n_rows + if rest > 0 { 1 } else { 0 };
        let cell_size = (max_size + spacing) / Vec2::new(n_rows as f32, n_cols as f32);

        self.grid = Some((n_rows, n_cols));

        ui.columns(n_cols, |cols| {
            for (i, entry) in self.entries.iter().enumerate() {
                let col = &mut cols[i % n_cols];
                col.vertical_centered(|ui| {
                    let label = entry.file_name().unwrap().to_str().unwrap();
                    let thumb = self.find_thumbnail(entry);
                    self.draw_thumbnail(ui, label, thumb.as_ref(), cell_size, self.cursor == i);
                });
            }
        });
    }

    pub fn draw(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        CentralPanel::default().show_inside(ui, |ui| {
            ScrollArea::both().show(ui, |ui| {
                self.draw_thumbnails(ui);
            });
        });
    }
}
