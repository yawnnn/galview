use core::str;
use egui::{Key, Vec2};
use std::{
    borrow::Cow,
    cmp, env, iter,
    path::{Path, PathBuf},
};

use crate::{
    common::{NumExt, frame_start},
    ui::UiExt,
};

/// compare strings with awareness of number substrings
fn smart_str_cmp(a: &str, b: &str) -> cmp::Ordering {
    fn take_number(chars: &mut iter::Peekable<str::Chars>) -> u64 {
        let mut n: u64 = 0;
        while let Some(&c) = chars.peek()
            && c.is_ascii_digit()
        {
            n = n.saturating_mul(10).saturating_add(c as u64 - '0' as u64);
            chars.next();
        }
        n
    }

    let mut a_chars = a.chars().peekable();
    let mut b_chars = b.chars().peekable();

    loop {
        match (a_chars.peek(), b_chars.peek()) {
            (None, None) => return cmp::Ordering::Equal,
            (None, Some(_)) => return cmp::Ordering::Less,
            (Some(_), None) => return cmp::Ordering::Greater,
            (Some(&ac), Some(&bc)) => {
                if ac.is_ascii_digit() && bc.is_ascii_digit() {
                    let a_num = take_number(&mut a_chars);
                    let b_num = take_number(&mut b_chars);
                    match a_num.cmp(&b_num) {
                        cmp::Ordering::Equal => continue,
                        other => return other,
                    }
                } else {
                    match ac.cmp(&bc) {
                        cmp::Ordering::Equal => {
                            a_chars.next();
                            b_chars.next();
                        }
                        other => return other,
                    }
                }
            }
        }
    }
}

#[derive(Default)]
pub struct App {
    pub pics_dir: PathBuf,
    pub res_dir: PathBuf,
    pub window_size: Vec2,
    pub grid_size: Vec2,
    pub entries: Vec<PathBuf>,
    pub cursor: usize,
}

impl App {
    pub fn new() -> Self {
        let home = PathBuf::from(env::var("HOME").unwrap());
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or("..".into()));
        let res_dir = manifest_dir.join("resources");
        let pics_dir = home.join("Pictures");
        let entries = Self::read_dir(&pics_dir);
        App {
            res_dir,
            pics_dir,
            entries,
            ..Default::default()
        }
    }

    pub fn res_path(&self, name: impl AsRef<Path>) -> PathBuf {
        self.res_dir.join(name)
    }

    pub fn read_dir(dir: &Path) -> Vec<PathBuf> {
        let mut entries: Vec<_> = dir
            .read_dir()
            .unwrap_or_else(|e| panic!("{:?}: {e}", dir))
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();
        entries.sort_by(|a, b| {
            let a = a.file_name().unwrap().to_string_lossy();
            let b = b.file_name().unwrap().to_string_lossy();
            smart_str_cmp(&a, &b)
        });

        entries
    }

    pub fn find_folder_thumbnail(folder: &Path) -> Option<PathBuf> {
        for e in folder.read_dir().ok()? {
            let e = e.ok()?;
            if e.metadata().unwrap().is_file() {
                return Some(e.path());
            }
        }

        for e in folder.read_dir().ok()? {
            let e = e.ok()?;
            if e.metadata().unwrap().is_dir()
                && let Some(p) = Self::find_folder_thumbnail(&e.path())
            {
                return Some(p);
            }
        }

        None
    }

    pub fn find_thumbnail<'a>(&self, entry: &'a PathBuf) -> Cow<'a, Path> {
        if entry.metadata().unwrap().is_file() {
            entry.into()
        } else if let Some(p) = Self::find_folder_thumbnail(entry) {
            p.into()
        } else {
            self.res_path("folder.svg").into()
        }
    }

    pub fn change_dir(&mut self, new_dir: PathBuf) {
        self.pics_dir = new_dir;
        self.entries = Self::read_dir(&self.pics_dir);
        self.cursor = 0;
    }

    pub fn move_cursor(&mut self, key: &Key) {
        let max = self.entries.len();
        self.cursor = match key {
            Key::ArrowLeft => self.cursor.modulo_sub(1, max),
            Key::ArrowRight => self.cursor.modulo_add(1, max),
            Key::ArrowUp => self.cursor.modulo_sub(self.grid_size.y as usize, max),
            Key::ArrowDown => self.cursor.modulo_add(self.grid_size.y as usize, max),
            _ => unreachable!(),
        };
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
            Key::Backspace => self.change_dir(self.pics_dir.parent().unwrap().into()),
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
