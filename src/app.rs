use egui::{Key, Vec2};
use std::{
    borrow::Cow,
    env,
    path::{Path, PathBuf},
};

use crate::{common::{NumExt, frame_start}, ui::UiExt};

#[derive(Default)]
pub struct App {
    pub window_size: Vec2,
    pub root: PathBuf,
    pub resources_dir: PathBuf,
    pub entries: Vec<PathBuf>,
    pub cursor: usize,
    pub grid: Option<(usize, usize)>,
}

impl App {
    pub fn new() -> Self {
        let home = PathBuf::from(env::var("HOME").unwrap());
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let resources_dir = manifest_dir.join("resources");
        let root = home.join("Pictures");
        let entries = Self::read_dir(&root);
        App {
            resources_dir,
            root,
            entries,
            ..Default::default()
        }
    }

    pub fn res_path(&self, name: impl AsRef<Path>) -> PathBuf {
        self.resources_dir.join(name)
    }

    pub fn read_dir(dir: &Path) -> Vec<PathBuf> {
        dir.read_dir()
            .unwrap_or_else(|e| panic!("{:?}: {e}", dir))
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect()
    }

    pub fn folder_find_thumbnail(folder: &Path) -> Option<PathBuf> {
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

    pub fn find_thumbnail<'a>(&self, entry: &'a PathBuf) -> Cow<'a, Path> {
        if entry.metadata().unwrap().is_file() {
            entry.into()
        } else if let Some(p) = Self::folder_find_thumbnail(entry) {
            p.into()
        } else {
            self.res_path("folder.svg").into()
        }
    }

    pub fn change_dir(&mut self, new_dir: PathBuf) {
        self.root = new_dir;
        self.entries = Self::read_dir(&self.root);
        self.cursor = 0;
    }

    pub fn move_cursor(&mut self, key: &Key) {
        let max = self.entries.len();
        match key {
            Key::ArrowLeft => self.cursor = self.cursor.modulo_sub(1, max),
            Key::ArrowRight => self.cursor = self.cursor.modulo_add(1, max),
            Key::ArrowUp if let Some(grid) = self.grid => {
                self.cursor = self.cursor.modulo_sub(grid.1, max)
            }
            Key::ArrowDown if let Some(grid) = self.grid => {
                self.cursor = self.cursor.modulo_add(grid.1, max)
            }
            _ => unreachable!(),
        }
    }

    pub fn handle_input(&mut self, ui: &mut egui::Ui) {
        ui.keys_pressed().iter().for_each(|key| match key {
            Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown => {
                self.move_cursor(key)
            }
            Key::Enter if let Some(entry) = self.entries.get(self.cursor) => {
                if entry.is_file() {
                    todo!()
                } else {
                    self.change_dir(entry.clone());
                }
            }
            Key::Backspace => self.change_dir(self.root.parent().unwrap().into()),
            Key::Q => ui.send_viewport_cmd(egui::ViewportCommand::Close),
            _ => (),
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        frame_start();
        self.window_size = ui.available_size();
        self.draw(ui, frame);
        self.handle_input(ui);
    }
}