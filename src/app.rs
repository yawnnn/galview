use core::str;
use egui::{Key, Vec2};

use std::{
    borrow::Cow,
    cmp, env, iter, mem,
    path::{Path, PathBuf},
};

use crate::{common::*, logging::*, ui::UiExt};

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
            (Some(ac), Some(bc)) if ac.is_ascii_digit() && bc.is_ascii_digit() => {
                let a_num = take_number(&mut a_chars);
                let b_num = take_number(&mut b_chars);
                match a_num.cmp(&b_num) {
                    cmp::Ordering::Equal => continue,
                    ord => return ord,
                }
            }
            (Some(ac), Some(bc)) => match ac.cmp(bc) {
                cmp::Ordering::Equal => {
                    a_chars.next();
                    b_chars.next();
                }
                ord => return ord,
            },
        }
    }
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

enum Direction {
    Up,
    Down,
}

fn nav_dir(directory: &mut PathBuf, direction: Direction) -> Vec<PathBuf> {
    let mut dir = directory.to_owned();
    let mut entries: Vec<_>;

    loop {
        if matches!(direction, Direction::Up) {
            println!("reading {dir:?}");
        }
        entries = read_dir(&dir);
        if entries.len() != 1 || entries[0].is_file() {
            break;
        }
        println!("looping");
        match direction {
            Direction::Up => dir = dir.parent().unwrap().to_owned(),
            Direction::Down => dir = entries.pop().unwrap(),
        }
    }

    *directory = dir;

    entries
}

pub fn find_folder_thumbnail(folder: &Path) -> Option<PathBuf> {
    let mut entries = read_dir(folder);
    if let Some(e) = entries.iter_mut().find(|e| e.is_file()) {
        return Some(mem::take(e));
    }

    entries
        .iter()
        .find(|e| e.is_dir())
        .and_then(|e| find_folder_thumbnail(e))
}

#[derive(Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Dirs,
    Files,
}

#[derive(Default)]
pub struct App {
    pub res_dir: PathBuf,
    pub window_size: Vec2,

    // UI state
    pub mode: Mode,
    pub curr_dir: PathBuf,
    pub curr_entries: Vec<PathBuf>,
    pub curr_entry: usize,

    // UI cache
    pub grid_size: Vec2,
    pub prev_entry: usize,
}

impl App {
    pub fn new() -> Self {
        let home = PathBuf::from(env::var("HOME").unwrap());
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or("..".into()));
        let res_dir = manifest_dir.join("resources");
        let curr_dir = home.join("Pictures");
        let curr_entries = read_dir(&curr_dir);
        App {
            res_dir,
            curr_dir,
            curr_entries,
            prev_entry: usize::MAX,
            ..Default::default()
        }
    }

    pub fn curr_entry(&self) -> Option<&PathBuf> {
        self.curr_entries.get(self.curr_entry)
    }

    pub fn res_path(&self, name: impl AsRef<Path>) -> PathBuf {
        self.res_dir.join(name)
    }

    pub fn find_thumbnail<'a>(&self, entry: &'a Path) -> Cow<'a, Path> {
        if entry.is_file() {
            Cow::from(entry)
        } else if let Some(p) = find_folder_thumbnail(entry) {
            p.into()
        } else {
            self.res_path("folder.svg").into()
        }
    }

    pub fn set_pics_dir(&mut self, ctx: &egui::Context, dir: PathBuf, entries: Vec<PathBuf>) {
        // // forget old images
        // for e in &self.curr_entries {
        //     ctx.forget_image(&self.find_thumbnail(Cow::from(e)).to_uri());
        // }
        ctx.forget_all_images();
        ctx.request_repaint();
        self.curr_dir = dir;
        self.curr_entries = entries;
        self.curr_entry = 0;
    }

    pub fn change_mode(&mut self, ctx: &egui::Context, mode: Mode) {
        if self.mode == mode {
            return;
        }
        ctx.forget_all_images();
        ctx.request_repaint(); // after i enabled animation (it shouldn't make a difference anyways cause i dropped everything before)
        self.mode = mode;
    }

    pub fn nav_forward(&mut self, ctx: &egui::Context, target: &Path) {
        if target.is_dir() {
            let mut dir = target.to_owned();
            let entries = nav_dir(&mut dir, Direction::Down);

            // if all i have left is files, go into file mode
            let new_mode = if !entries.is_empty() && entries.iter().all(|e| e.is_file()) {
                Mode::Files
            } else {
                Mode::Dirs
            };
            self.change_mode(ctx, new_mode);
            self.set_pics_dir(ctx, dir, entries);
        } else if target.is_file() {
            self.change_mode(ctx, Mode::Files);
        } else {
            panic!("File or directory {target:?} not found");
        }
    }

    pub fn nav_backwards(&mut self, ctx: &egui::Context) {
        let mut dir = self.curr_dir.parent().unwrap().to_owned();
        let entries = nav_dir(&mut dir, Direction::Up);
        if self.mode == Mode::Files {
            self.change_mode(ctx, Mode::Dirs);
        }
        self.set_pics_dir(ctx, dir, entries);
    }

    pub fn move_cursor(&mut self, key: &Key) {        
        let Some(original) = self.curr_entry().cloned() else {
            return;
        };
        let max = self.curr_entries.len();
        loop {
            self.curr_entry = match key {
                Key::ArrowLeft => self.curr_entry.modulo_sub(1, max),
                Key::ArrowRight => self.curr_entry.modulo_add(1, max),
                Key::ArrowUp => self.curr_entry.modulo_sub(self.grid_size.x as usize, max),
                Key::ArrowDown => self.curr_entry.modulo_add(self.grid_size.x as usize, max),
                _ => unreachable!(),
            };

            let new_curr = self.curr_entry().unwrap();
            // if in Files mode, loop around
            if self.mode == Mode::Dirs
                || new_curr.is_file()
                || *new_curr == original
            {
                break;
            }
        }
    }

    pub fn handle_input(&mut self, ui: &mut egui::Ui) {
        ui.keys_pressed().iter().for_each(|key| match key {
            Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown => {
                self.move_cursor(key)
            }
            Key::Enter if let Some(entry) = self.curr_entry() => {
                self.nav_forward(ui, &entry.clone());
            }
            Key::Backspace | Key::Escape => self.nav_backwards(ui),
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
