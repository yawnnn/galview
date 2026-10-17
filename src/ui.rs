use eframe::egui;
use egui::{
    CentralPanel, Color32, Event, Image, Key, Label, Panel, Response, RichText,
    ScrollArea, Ui, Vec2,
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

const MAX_COLS: usize = 5;
const MAX_ROWS: usize = 3;

#[allow(unused)]
#[derive(Clone, Copy, Default)]
struct GridLayout {
    cols: usize,
    rows: usize,
    size: Vec2, // x: cols y: rows
    img_size: Vec2,
    label_size: f32,
}

impl GridLayout {
    fn new(
        ui: &Ui,
        max_cols: usize,
        max_rows: usize,
        cells: usize,
    ) -> GridLayout {
        let cells = cells.min(max_cols * max_rows);
        if cells == 0 {
            return GridLayout::default();
        }
        let spacing = ui.spacing().item_spacing;
        let cols = cells.min(max_cols); // prioritize laying out columns
        let rows = cells.div_ceil(cols);
        let size = Vec2::new(cols as f32, rows as f32);
        let available = ui.available_size();
        let label_size = ui.text_style_height(&egui::TextStyle::Body);
        // window_margin is a smidgeon too much, ui.spacing().item_spacing.y is too little
        let label_tot =
            label_size + ui.style().spacing.window_margin.bottom as f32 - 1.;
        let mut cell_size = spacing * size.sub(Vec2::ONE);
        cell_size.y += label_tot * size.y;
        let img_size = ((available - cell_size) / size).max(Vec2::ZERO);

        GridLayout {
            cols,
            rows,
            size,
            img_size,
            label_size,
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
        ui.set_width(grid.img_size.x);

        let img_rect =
            egui::Rect::from_min_size(ui.cursor().min, grid.img_size);
        let resp = if ui.is_rect_visible(img_rect) {
            self.draw_image(ui, thumb, grid.img_size)
        } else {
            ui.allocate_response(grid.img_size, egui::Sense::hover())
        };

        let hover_text = thumb
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy();
        let resp = resp.on_hover_text_at_pointer(hover_text);

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

        let label = entry.file_name().unwrap().to_string_lossy();
        ui.add(Label::new(
            RichText::new(label)
                .size(grid.label_size)
                .color(Color32::GRAY)
                .strong(),
        ));
    }

    fn draw_grid(&mut self, ui: &mut Ui) {
        let grid = GridLayout::new(ui, MAX_COLS, MAX_ROWS, self.entries.len());

        self.dim_x = grid.size.x as usize;
        self.dim_y = grid.size.y as usize;
        let mut prev_entry = self.prev_frame_cursor;

        egui::Grid::new("thumb_grid")
            .num_columns(grid.cols)
            .show(ui, |ui| {
                for (i, entry) in self.entries.iter().enumerate() {
                    ui.vertical_centered(|ui| {
                        let thumb = self.find_thumbnail(entry);
                        let hovered = self.cursor == i;
                        // calling scroll_to_me every time, causes a weird glitch
                        let scroll_to_me = hovered && self.cursor != prev_entry;
                        if scroll_to_me {
                            prev_entry = self.cursor;
                        }
                        self.draw_entry(
                            ui,
                            entry,
                            &thumb,
                            &grid,
                            hovered,
                            scroll_to_me,
                        );
                    });

                    if (i + 1) % grid.cols == 0 {
                        ui.end_row();
                    }
                }
            });

        self.prev_frame_cursor = prev_entry;
    }

    fn draw_dirs(&mut self, ui: &mut Ui) {
        ScrollArea::both().show(ui, |ui| {
            self.draw_grid(ui);
        });
    }

    fn draw_files(&mut self, ui: &mut Ui) {
        let num_entries = self.entries.len();
        let grid = GridLayout::new(ui, 1, 1, 1);
        let mut available_size = Vec2::ZERO;

        ui.vertical_centered(|ui| {
            available_size = ui.available_size();
            self.draw_image(ui, self.curr_entry().unwrap(), grid.img_size);
            ui.add(Label::new(
                RichText::new(format!("{} of {num_entries}", self.cursor + 1))
                    .size(grid.label_size)
                    .color(Color32::GRAY)
                    .strong(),
            ));
        });

        // preload neighboring entries, forget the others
        for (i, p) in self.entries.iter().enumerate() {
            let uri = p.to_uri();
            match self.cursor.abs_diff(i) {
                1..=3 => {
                    let _ = Image::from_uri(uri)
                        .fit_to_exact_size(grid.size)
                        .load_for_size(ui.ctx(), available_size);
                }
                4.. => ui.forget_image(&uri),
                _ => (),
            }
        }
    }

    fn draw_filter(&mut self, ui: &mut Ui) {
        let mut text = self.get_filter();

        Panel::bottom("filter").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("filter")
                        .text_style(egui::TextStyle::Heading),
                );
                let response = ui.add(
                    egui::TextEdit::singleline(&mut text)
                        .desired_width(f32::INFINITY),
                );

                response.request_focus();

                if response.changed() {
                    self.update_filter(text);
                }
            });
        });
    }

    pub fn draw(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if self.filter_on {
            self.draw_filter(ui);
        }

        CentralPanel::default().show(ui, |ui| {
            match self.mode {
                Mode::Dirs => self.draw_dirs(ui),
                Mode::Files => self.draw_files(ui),
            };
        });
    }
}
