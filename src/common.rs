use core::str;
use std::{
    cmp, iter,
    path::{Path, PathBuf},
};

pub trait NumExt {
    type Signed;

    /// saturating_add with custom upper ceiling
    fn saturating_add_on(self, rhs: Self::Signed, on: Self) -> Self;

    /// saturating add with custom upper cealing, but it saturates to the current value, not the ceiling
    fn saturating_add_on_or_stay(self, rhs: Self::Signed, on: Self) -> Self;

    /// wrapping_add with custom upper ceiling
    fn wrapping_add_on(self, rhs: Self::Signed, on: Self) -> Self;
}

impl NumExt for usize {
    type Signed = isize;

    fn saturating_add_on(self, rhs: Self::Signed, on: Self) -> Self {
        match self as Self::Signed + rhs {
            ..0 => 0,
            res if res as Self > on => on,
            res => res as Self,
        }
    }

    fn saturating_add_on_or_stay(self, rhs: Self::Signed, on: Self) -> Self {
        match self as Self::Signed + rhs {
            ..0 => self,
            res if res as Self > on => self,
            res => res as Self,
        }
    }

    fn wrapping_add_on(self, rhs: Self::Signed, on: Self) -> Self {
        (self as Self::Signed + rhs).rem_euclid(on as isize) as usize
    }
}

pub trait PathExt {
    fn to_uri(&self) -> String;
}

impl PathExt for Path {
    fn to_uri(&self) -> String {
        format!("file://{}", self.to_str().unwrap())
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

/// read dir without complications
pub fn read_dir(dir: &Path) -> impl Iterator<Item = PathBuf> + use<> {
    dir.read_dir()
        .unwrap_or_else(|e| panic!("{:?}: {e}", dir))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
}

/// read dir entries and sort them
pub fn dir_entries_sorted(dir: &Path) -> Vec<PathBuf> {
    let mut entries: Vec<_> = read_dir(dir).collect();
    entries.sort_by(|a, b| {
        let a = a.file_name().unwrap().to_string_lossy();
        let b = b.file_name().unwrap().to_string_lossy();
        natural_cmp(&a, &b)
    });
    entries
}

use egui_extras::loaders::webp_loader;

// hack to enable animations
pub fn enable_animations(ctx: &egui::Context) {
    if !ctx.is_loader_installed(webp_loader::WebPLoader::ID) {
        let webp_loader = webp_loader::WebPLoader::default();
        ctx.add_image_loader(std::sync::Arc::new(webp_loader));
    }
}

// hack to disable animations: remove the only loader that loads them (so far)
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
