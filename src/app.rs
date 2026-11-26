use egui::{Key, Modifiers, Vec2};
use std::{
    borrow::Cow,
    collections::HashMap,
    env, fs,
    io::Write,
    mem,
    path::{Path, PathBuf},
    rc::Rc,
    sync::LazyLock,
};

use crate::{common::*, logging::*, ui::UiExt};

enum Direction {
    Up,
    Down,
}

fn nav_towards(
    target_dir: &mut PathBuf,
    direction: Direction,
) -> Vec<Rc<PathBuf>> {
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

    dir_entries_sorted_rc(target_dir)
}

pub fn find_folder_thumbnail(folder: &Path) -> Option<PathBuf> {
    let mut entries = dir_entries_sorted(folder);
    if let Some(e) = entries.iter_mut().find(|e| e.is_file()) {
        return Some(mem::take(e));
    }

    entries
        .iter()
        .find(|e| e.is_dir())
        .and_then(|e| find_folder_thumbnail(e))
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum ResID {
    FolderSvg,
}

type Resouces<T> = HashMap<ResID, T>;

macro_rules! bundle_resource {
    ($id:expr, $flname:literal) => {
        (
            $id,
            (
                $flname,
                include_bytes!(concat!("../resources/", $flname)).as_slice(),
            ),
        )
    };
}

static RESOURCE_DATA: LazyLock<Resouces<(&'static str, &'static [u8])>> =
    LazyLock::new(|| {
        HashMap::from([bundle_resource!(ResID::FolderSvg, "folder.svg")])
    });

/// write bundled resources to disk, so i always return a path from find_thumbnail
/// it's just easier for now
fn init_resources(home: &Path) -> Resouces<PathBuf> {
    let xdg_cache = env::var("XDB_CACHE_HOME").unwrap_or_default();
    let xdg_cache = if xdg_cache.is_empty() {
        home.join(".cache")
    } else {
        PathBuf::from(xdg_cache)
    };
    let res_dir = xdg_cache.join(env!("CARGO_PKG_NAME")).join("res");
    fs::create_dir_all(&res_dir).unwrap_or_else(|e| {
        panic!("Couldn't create resources directory {res_dir:?}: {e}")
    });
    let mut resources = HashMap::new();
    for (id, (name, bytes)) in RESOURCE_DATA.iter() {
        let p = res_dir.join(name);
        if !fs::exists(&p).unwrap_or(false) {
            fs::File::create(&p)
                .and_then(|mut fl| fl.write_all(bytes))
                .unwrap_or_else(|e| {
                    panic!("Couldn't access resource {p:?}: {e}")
                });
        }
        resources.insert(*id, p);
    }
    resources
}

#[derive(Clone, Copy)]
pub struct YesNoDialog {
    pub curr: bool,
}

impl YesNoDialog {
    pub fn new(start: bool) -> Self {
        YesNoDialog { curr: start }
    }

    pub fn toggle(mut self) -> Self {
        self.curr = !self.curr;
        self
    }
}

#[derive(Default)]
pub enum Dialogs {
    #[default]
    None,
    Filter,
    Delete(YesNoDialog),
}

#[derive(Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Dirs,
    Files,
}

#[derive(Default)]
pub struct App {
    resources: Resouces<PathBuf>,
    pub window_size: Vec2,

    pub mode: Mode,
    pub dir: PathBuf,
    raw_entries: Vec<Rc<PathBuf>>,
    pub entries: Vec<Rc<PathBuf>>,
    pub cursor: usize,
    pub dim_x: usize,
    pub dim_y: usize,
    pub prev_frame_cursor: usize,
    nav_stack: Vec<PathBuf>,

    pub dialogs: Dialogs,
    filter_text: String,
}

impl App {
    pub fn new() -> Self {
        let home = PathBuf::from(env::var("HOME").unwrap());
        let dir = home.join("Pictures");
        let resources = init_resources(&home);
        let raw_entries = dir_entries_sorted_rc(&dir);
        let entries = raw_entries.clone();
        App {
            resources,
            dir,
            raw_entries,
            entries,
            prev_frame_cursor: usize::MAX,
            ..Default::default()
        }
    }

    fn has_filter(&self) -> bool {
        self.mode == Mode::Dirs && !self.filter_text.is_empty()
    }

    fn passes_filter(&self, e: &Path) -> bool {
        e.file_name()
            .unwrap()
            .to_string_lossy()
            .to_lowercase()
            .contains(&self.filter_text)
    }

    pub fn entries_update(&mut self) {
        self.entries = if self.has_filter() {
            self.raw_entries
                .iter()
                .filter(|e| self.passes_filter(e))
                .cloned()
                .collect()
        } else {
            self.raw_entries.clone()
        };

        self.cursor = 0;
    }

    pub fn entry_get(&self, idx: usize) -> Option<&PathBuf> {
        self.entries.get(idx).map(|e| e.as_ref())
    }

    pub fn entry_curr(&self) -> Option<&PathBuf> {
        self.entry_get(self.cursor)
    }

    pub fn entries_refresh(&mut self) {
        self.raw_entries = dir_entries_sorted_rc(&self.dir);
        self.entries_update();
    }

    fn res_path(&self, id: ResID) -> &Path {
        self.resources.get(&id).unwrap()
    }

    pub fn find_thumbnail<'p>(&self, entry: &'p Path) -> Cow<'p, Path> {
        if entry.is_file() {
            Cow::from(entry)
        } else if let Some(p) = find_folder_thumbnail(entry) {
            p.into()
        } else {
            self.res_path(ResID::FolderSvg).to_owned().into()
        }
    }

    fn pics_dir_set(&mut self, dir: PathBuf, entries: Vec<Rc<PathBuf>>) {
        if self.dir == dir {
            return;
        }
        self.dir = dir;
        self.raw_entries = entries;
        self.entries_update();
    }

    fn mode_set(&mut self, ctx: &egui::Context, mode: Mode) {
        if self.mode == mode {
            return;
        }
        match mode {
            Mode::Files => animations_enable(ctx),
            Mode::Dirs => animations_disable(ctx),
        }
        ctx.forget_all_images();
        ctx.request_repaint(); // after i enabled animation (it shouldn't make a difference anyways cause i dropped everything before)
        self.mode = mode;
    }

    fn nav_forward(&mut self, ctx: &egui::Context, target: &Path) {
        if let Some(e) = self.entry_curr().cloned() {
            self.nav_stack.push(e);
        }

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
            self.mode_set(ctx, new_mode);
            self.pics_dir_set(dir, entries);
        } else if target.is_file() {
            self.mode_set(ctx, Mode::Files);
        } else {
            panic!("File or directory {target:?} not found");
        }
    }

    fn nav_backward(&mut self, ctx: &egui::Context) {
        let mut dir = self.dir.parent().unwrap().to_owned();
        let entries = nav_towards(&mut dir, Direction::Up);
        self.mode_set(ctx, Mode::Dirs);
        self.pics_dir_set(dir, entries);

        if let Some(e) = self.nav_stack.pop()
            && let Some(pos) = self
                .entries
                .iter()
                .position(|e2| e2.as_path() == e.as_path())
        {
            self.cursor = pos;
        }
    }

    fn nav_enter(&mut self, ui: &mut egui::Ui) {
        if let Some(entry) = self.entry_curr() {
            self.nav_forward(ui, &entry.clone());
        }
    }

    fn cursor_move(&mut self, dir_x: isize, dir_y: isize) {
        if self.entries.is_empty() {
            return;
        };

        self.cursor = match self.mode {
            Mode::Dirs => self
                .cursor
                .saturating_add_on(dir_x, self.entries.len() - 1)
                .saturating_add_on_or_stay(
                    dir_y * self.dim_x as isize,
                    self.entries.len() - 1,
                ),
            Mode::Files => {
                self.cursor.wrapping_add_on(dir_x, self.entries.len())
            }
        };
    }

    fn filter_open(&mut self) {
        if self.mode == Mode::Files {
            return;
        }
        if matches!(self.dialogs, Dialogs::Delete { .. }) {
            return;
        }
        self.dialogs = Dialogs::Filter;
    }

    pub fn filter_get(&self) -> String {
        self.filter_text.clone()
    }

    pub fn filter_update(&mut self, text: String) {
        self.filter_text = text.to_lowercase();
        self.entries_update();
    }

    fn move_to_trash_curr(&mut self) {
        let Some(p) = self.entry_curr() else { return };
        match trash::delete(p) {
            Ok(_) => self.entries_refresh(),
            Err(e) => eprintln!("Failed to delete {}: {}", p.display(), e),
        }
    }

    fn remove_curr(&mut self) {
        let p = self.entry_curr().unwrap();
        let e = if p.is_dir() {
            fs::remove_dir_all(p)
        } else {
            fs::remove_file(p)
        };
        match e {
            Ok(_) => self.entries_refresh(),
            Err(e) => eprintln!("Failed to delete {}: {}", p.display(), e),
        }
    }

    fn ask_remove_curr(&mut self) {
        let Some(entry) = self.entry_curr() else {
            return;
        };

        if is_empty_dir(entry) {
            self.remove_curr();
        } else {
            self.dialogs = Dialogs::Delete(YesNoDialog::new(true));
        }
    }

    fn handle_key_filter(
        &mut self,
        ui: &mut egui::Ui,
        key: Key,
        _mods: Modifiers,
    ) {
        match key {
            Key::Enter => self.dialogs = Dialogs::None,
            Key::Escape => {
                self.dialogs = Dialogs::None;
                self.filter_update(String::new());
            }
            Key::ArrowUp | Key::ArrowDown => {
                self.dialogs = Dialogs::None;
                self.handle_key(ui, key, Modifiers::NONE)
            }
            _ => (),
        }
    }

    fn handle_key_delete(
        &mut self,
        dialog: YesNoDialog,
        _ui: &mut egui::Ui,
        key: Key,
        _mods: Modifiers,
    ) {
        match key {
            Key::Enter => {
                if dialog.curr {
                    self.remove_curr();
                }
                self.dialogs = Dialogs::None;
            }
            Key::Escape => self.dialogs = Dialogs::None,
            Key::ArrowLeft | Key::ArrowRight => {
                self.dialogs = Dialogs::Delete(dialog.toggle())
            }
            _ => (),
        }
    }

    fn handle_key(&mut self, ui: &mut egui::Ui, key: Key, mods: Modifiers) {
        match key {
            Key::ArrowLeft => self.cursor_move(-1, 0),
            Key::ArrowRight => self.cursor_move(1, 0),
            Key::ArrowUp => self.cursor_move(0, -1),
            Key::ArrowDown => self.cursor_move(0, 1),
            Key::Home => self.cursor = 0,
            Key::End => self.cursor = self.entries.len() - 1,
            Key::Enter => self.nav_enter(ui),
            Key::Backspace | Key::Escape => self.nav_backward(ui),
            Key::Slash => self.filter_open(),
            Key::Delete if mods.shift => self.ask_remove_curr(),
            Key::Delete => self.move_to_trash_curr(),
            Key::R => self.entries_refresh(),
            Key::Q => ui.send_viewport_cmd(egui::ViewportCommand::Close),
            _ => (),
        }
    }

    fn handle_input(&mut self, ui: &mut egui::Ui) {
        for (key, mods) in ui.keys_pressed() {
            match self.dialogs {
                Dialogs::Filter => self.handle_key_filter(ui, key, mods),
                Dialogs::Delete(dialog) => {
                    self.handle_key_delete(dialog, ui, key, mods)
                }
                _ => self.handle_key(ui, key, mods),
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
