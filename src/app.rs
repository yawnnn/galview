use core::str;
use egui::{Key, Vec2};

use std::{
    borrow::Cow,
    cmp, env, iter, mem,
    path::{Path, PathBuf},
};

use crate::{common::*, logging::*, ui::UiExt};

use egui_extras::loaders::webp_loader;

pub fn enable_animations(ctx: &egui::Context) {
    if !ctx.is_loader_installed(webp_loader::WebPLoader::ID) {
        let webp_loader = webp_loader::WebPLoader::default();
        ctx.add_image_loader(std::sync::Arc::new(webp_loader));
    }
}

pub fn disable_animations(ctx: &egui::Context) {
    let loaders = ctx.loaders();
    let mut img_loaders = loaders.image.lock();
    if let Some(pos) = img_loaders
        .iter()
        .position(|loader| loader.id() == webp_loader::WebPLoader::ID)
    {
        img_loaders.remove(pos);
    }
}

/// compare strings with awareness of number substrings
fn natural_cmp(a: &str, b: &str) -> cmp::Ordering {
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
            (Some(ac), Some(bc))
                if ac.is_ascii_digit() && bc.is_ascii_digit() =>
            {
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

fn read_dir(dir: &Path) -> impl Iterator<Item = PathBuf> + use<> {
    dir.read_dir()
        .unwrap_or_else(|e| panic!("{:?}: {e}", dir))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
}

fn read_dir_sorted(dir: &Path) -> Vec<PathBuf> {
    let mut entries: Vec<_> = read_dir(dir).collect();
    entries.sort_by(|a, b| {
        let a = a.file_name().unwrap().to_string_lossy();
        let b = b.file_name().unwrap().to_string_lossy();
        natural_cmp(&a, &b)
    });
    entries
}

enum Direction {
    Up,
    Down,
}

fn nav_towards(target_dir: &mut PathBuf, direction: Direction) -> Vec<PathBuf> {
    let mut dir = Cow::from(target_dir.as_path());

    loop {
        let mut it = read_dir(&dir);
        let Some(first) =
            it.next().filter(|e| !e.is_file() && it.next().is_none())
        else {
            break;
        };
        let new_dir = match direction {
            Direction::Up => dir.parent().unwrap(),
            Direction::Down => &first,
        };
        dir = new_dir.to_owned().into();
    }

    *target_dir = dir.to_path_buf();

    read_dir_sorted(target_dir)
}

pub fn find_folder_thumbnail(folder: &Path) -> Option<PathBuf> {
    let mut entries = read_dir_sorted(folder);
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
    pub dir: PathBuf,
    pub entries: Vec<PathBuf>,
    pub entry_idx: usize,

    // UI cache
    pub dim_x: usize,
    pub dim_y: usize,
    pub prev_entry: usize,
}

impl App {
    pub fn new() -> Self {
        let home = PathBuf::from(env::var("HOME").unwrap());
        let manifest_dir = PathBuf::from(
            env::var("CARGO_MANIFEST_DIR").unwrap_or("..".into()),
        );
        let res_dir = manifest_dir.join("resources");
        let dir = home.join("Pictures");
        let entries = read_dir_sorted(&dir);
        App {
            res_dir,
            dir,
            entries,
            prev_entry: usize::MAX,
            ..Default::default()
        }
    }

    pub fn curr_entry(&self) -> Option<&PathBuf> {
        self.entries.get(self.entry_idx)
    }

    pub fn entry(&self, idx: usize) -> Option<&PathBuf> {
        self.entries.get(idx)
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

    pub fn set_pics_dir(
        &mut self,
        ctx: &egui::Context,
        dir: PathBuf,
        entries: Vec<PathBuf>,
    ) {
        // // forget old images
        // for e in &self.curr_entries {
        //     ctx.forget_image(&self.find_thumbnail(Cow::from(e)).to_uri());
        // }
        ctx.forget_all_images();
        ctx.request_repaint();
        self.dir = dir;
        self.entries = entries;
        self.entry_idx = 0;
    }

    pub fn set_mode(&mut self, ctx: &egui::Context, mode: Mode) {
        if self.mode == mode {
            return;
        }
        match mode {
            Mode::Files => enable_animations(ctx),
            Mode::Dirs => disable_animations(ctx),
        }
        ctx.forget_all_images();
        ctx.request_repaint(); // after i enabled animation (it shouldn't make a difference anyways cause i dropped everything before)
        self.mode = mode;
    }

    pub fn nav_forward(&mut self, ctx: &egui::Context, target: &Path) {
        if target.is_dir() {
            let mut dir = target.to_owned();
            let entries = nav_towards(&mut dir, Direction::Down);

            // if all i have left is files, go into file mode
            let new_mode =
                if !entries.is_empty() && entries.iter().all(|e| e.is_file()) {
                    Mode::Files
                } else {
                    Mode::Dirs
                };
            self.set_mode(ctx, new_mode);
            self.set_pics_dir(ctx, dir, entries);
        } else if target.is_file() {
            self.set_mode(ctx, Mode::Files);
        } else {
            panic!("File or directory {target:?} not found");
        }
    }

    pub fn nav_backward(&mut self, ctx: &egui::Context) {
        let mut dir = self.dir.parent().unwrap().to_owned();
        let entries = nav_towards(&mut dir, Direction::Up);
        self.set_mode(ctx, Mode::Dirs);
        self.set_pics_dir(ctx, dir, entries);
    }

    pub fn nav_entry(&mut self, dir_x: isize, dir_y: isize) {
        if self.entries.is_empty() {
            return;
        };

        self.entry_idx = match self.mode {
            Mode::Dirs => self
                .entry_idx
                .saturating_add_on(dir_x, self.entries.len() - 1)
                .saturating_add_on_or_stay(
                    dir_y * self.dim_x as isize,
                    self.entries.len() - 1
                ),
            Mode::Files => {
                self.entry_idx.wrapping_add_on(dir_x, self.entries.len())
            }
        };
    }

    fn nav_enter(&mut self, ui: &mut egui::Ui) {
        if let Some(entry) = self.curr_entry() {
            self.nav_forward(ui, &entry.clone());
        }
    }

    pub fn handle_input(&mut self, ui: &mut egui::Ui) {
        for key in ui.keys_pressed() {
            match key {
                Key::ArrowLeft => self.nav_entry(-1, 0),
                Key::ArrowRight => self.nav_entry(1, 0),
                Key::ArrowUp => self.nav_entry(0, -1),
                Key::ArrowDown => self.nav_entry(0, 1),
                Key::Enter => self.nav_enter(ui),
                Key::Backspace | Key::Escape => self.nav_backward(ui),
                Key::Q => ui.send_viewport_cmd(egui::ViewportCommand::Close),
                _ => (),
            }
        }
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
