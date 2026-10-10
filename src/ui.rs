use eframe::egui;
use egui::{
    CentralPanel, Color32, Event, Image, Key, Label, Response, RichText, ScrollArea, Ui, Vec2,
};
use std::{ops::Sub, path::Path};

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

const MAX_ROWS: usize = 4;
const MAX_COLS: usize = 4;

#[allow(unused)]
#[derive(Clone, Copy, Default)]
struct GridLayout {
    cols: usize,
    rows: usize,
    size: Vec2, // x: cols y: rows
    cell_size: Vec2,
    label_height: f32,
}

impl GridLayout {
    fn new(ui: &Ui, cells: usize) -> GridLayout {
        let cells = cells.min(MAX_COLS * MAX_ROWS);
        if cells == 0 {
            return GridLayout::default();
        }
        let spacing = ui.spacing().item_spacing;
        let cols = cells.min(MAX_COLS); // prioritize laying out columns
        let rows = cells.div_ceil(cols);
        let size = Vec2::new(cols as f32, rows as f32);
        let available = ui.available_size();
        let label_height = ui.text_style_height(&egui::TextStyle::Body);
        // window_margin is a smidgeon too much, ui.spacing().item_spacing.y is too little
        let label_size = label_height + ui.style().spacing.window_margin.bottom as f32 - 1.;
        let mut raw_cell_size = spacing * size.sub(Vec2::ONE);
        raw_cell_size.y += label_size * size.y;
        let cell_size = ((available - raw_cell_size) / size).max(Vec2::ZERO);

        GridLayout {
            cols,
            rows,
            size,
            cell_size,
            label_height,
        }
    }
}

impl App {
    fn draw_image(&self, ui: &mut Ui, path: &Path, size: Vec2) -> Response {
        let img = Image::from_uri(path.to_uri()).fit_to_exact_size(size);
        ui.add(img)
    }

    fn draw_entry(
        &self,
        ui: &mut Ui,
        entry: &Path,
        thumb: &Path,
        grid: &GridLayout,
        hovered: bool,
        scroll_to_me: bool,
    ) {
        ui.set_width(grid.cell_size.x);

        let hover_text = thumb
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy();
        let resp = self
            .draw_image(ui, thumb, grid.cell_size)
            .on_hover_text_at_pointer(hover_text);

        let label = entry.file_name().unwrap().to_string_lossy();
        ui.add(Label::new(
            RichText::new(label)
                .size(grid.label_height)
                .color(Color32::GRAY)
                .strong(),
        ));

        if hovered {
            ui.painter().rect_stroke(
                resp.rect,
                0.,
                egui::Stroke::new(3.0f32, egui::Color32::from_rgb(255, 200, 0)),
                egui::StrokeKind::Outside,
            );
        }

        if scroll_to_me {
            resp.scroll_to_me(Some(egui::Align::Center));
        }
    }

    fn draw_grid(&mut self, ui: &mut Ui) {
        let grid = GridLayout::new(ui, self.curr_entries.len());

        self.grid_size = grid.size;
        let mut prev_entry = self.prev_entry;

        egui::Grid::new("thumb_grid")
            .num_columns(grid.cols)
            .show(ui, |ui| {
                for (i, entry) in self.curr_entries.iter().enumerate() {
                    ui.vertical_centered(|ui| {
                        let thumb = self.find_thumbnail(entry);
                        let hovered = self.curr_entry == i;
                        // calling scroll_to_me every time, causes a weird glitch
                        let scroll_to_me = hovered && self.curr_entry != prev_entry;
                        if scroll_to_me {
                            prev_entry = self.curr_entry;
                        }
                        self.draw_entry(ui, entry, &thumb, &grid, hovered, scroll_to_me);
                    });

                    if (i + 1) % grid.cols == 0 {
                        ui.end_row();
                    }
                }
            });

        self.prev_entry = prev_entry;
    }

    fn draw_dirs(&mut self, ui: &mut Ui) {
        ScrollArea::both().show(ui, |ui| {
            self.draw_grid(ui);
        });
    }

    fn draw_files(&mut self, ui: &mut Ui) {
        ui.centered_and_justified(|ui| {
            self.draw_image(ui, self.curr_entry().unwrap(), ui.available_size());
        });
    }

    pub fn draw(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ui, |ui| {
            match self.mode {
                Mode::Dirs => self.draw_dirs(ui),
                Mode::Files => self.draw_files(ui),
            };
        });
    }
}
