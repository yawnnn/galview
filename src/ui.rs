use eframe::egui;
use egui::{
    CentralPanel, Color32, Event, Image, Key, Label, Response, RichText, ScrollArea, Ui, Vec2,
};
use std::{borrow::Cow, ops::Sub, path::Path};

use crate::{App, app::Mode, common::*};

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

const MAX_ROWS: usize = 5;
const MAX_COLS: usize = 5;

impl App {
    fn add_image(&self, ui: &mut Ui, path: &Path, size: Vec2) -> Response {
        let img = Image::from_uri(path.to_uri()).fit_to_exact_size(size);
        ui.add(img)
    }

    fn draw_entry(
        &self,
        ui: &mut Ui,
        entry: &Path,
        thumb: &Path,
        cell_size: Vec2,
        is_hovered: bool,
    ) {
        let hover_text = thumb
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy();
        let resp = self
            .add_image(ui, thumb, cell_size)
            .on_hover_text_at_pointer(hover_text);

        if is_hovered {
            //resp.scroll_to_me(Some(egui::Align::Center));
            ui.painter().rect_stroke(
                resp.rect,
                0.,
                egui::Stroke::new(3.0f32, egui::Color32::from_rgb(255, 200, 0)),
                egui::StrokeKind::Outside,
            );
            resp.scroll_to_me(Some(egui::Align::Center));
        }

        let label = entry.file_name().unwrap().to_string_lossy();
        ui.add(Label::new(
            RichText::new(label).size(15.).color(Color32::GRAY).strong(),
        ));
    }

    fn draw_grid(&mut self, ui: &mut Ui) {
        const MAX_CELLS: usize = MAX_ROWS * MAX_COLS;
        let n_cells = self.curr_entries.len().min(MAX_CELLS);
        if n_cells == 0 {
            return;
        }

        let spacing = ui.spacing().item_spacing;
        let available = ui.available_size();
        let n_cols = n_cells.min(MAX_COLS); // prioritize laying out columns
        let n_rows = n_cells.div_ceil(n_cols);
        let grid_size = Vec2::new(n_cols as f32, n_rows as f32);
        let label_height = 20.0; // 15pt font + some padding
        let mut total_spacing = spacing * grid_size.sub(Vec2::ONE);
        total_spacing.y -= label_height * n_rows as f32;
        let img_size = ((available - total_spacing) / grid_size).max(Vec2::ZERO);

        self.grid_size = grid_size;

        egui::Grid::new("thumb_grid")
            .num_columns(n_cols)
            .show(ui, |ui| {
                for (i, entry) in self.curr_entries.iter().enumerate() {
                    let is_hovered = self.curr_entry == i;

                    ui.vertical_centered(|ui| {
                        ui.set_width(img_size.x);
                        let thumb = self.find_thumbnail(Cow::from(entry));
                        self.draw_entry(ui, entry, &thumb, img_size, is_hovered);
                    });

                    if (i + 1) % n_cols == 0 {
                        ui.end_row();
                    }
                }
            });
    }

    fn draw_dirs(&mut self, ui: &mut Ui) {
        ScrollArea::both().show(ui, |ui| {
            self.draw_grid(ui);
        });
    }

    pub fn draw(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ui, |ui| {
            match self.mode {
                Mode::Dirs => self.draw_dirs(ui),
                Mode::Files => {
                    ui.centered_and_justified(|ui| {
                        self.add_image(ui, self.curr_entry().unwrap(), ui.available_size());
                    });
                }
            };
        });
    }
}
