use core::str;
use std::{
    cmp::{self, Ordering},
    iter,
    path::{Path, PathBuf},
    rc::Rc,
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

#[derive(PartialEq, Eq)]
struct NaturalChar(char);

impl From<char> for NaturalChar {
    fn from(value: char) -> Self {
        NaturalChar(value)
    }
}

impl PartialOrd for NaturalChar {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NaturalChar {
    fn cmp(&self, other: &Self) -> Ordering {
        let c1 = self.0;
        let c2 = other.0;

        // first special chars, then numbers, then the rest
        // TODO: Dolphin has a special ranking among special chars
        let mut r = c1.is_alphanumeric() as i32 - c2.is_alphanumeric() as i32;
        if r == 0 {
            r = c2.is_numeric() as i32 - c1.is_numeric() as i32; // NOTE: inverted
            if r == 0 {
                // group-by letter, then by case
                // lowercase comes before uppercase (Dolphin does this, i follow suit)
                return c1
                    .to_lowercase()
                    .cmp(c2.to_lowercase())
                    .then(c1.cmp(&c2).reverse());
            }
        }

        match r {
            1.. => Ordering::Greater,
            ..0 => Ordering::Less,
            0 => Ordering::Equal,
        }
    }
}

fn natural_str(s: &str) -> impl Iterator<Item = NaturalChar> {
    s.chars().map(NaturalChar::from)
}

fn cmp_num(s1: &str, s2: &str) -> Ordering {
    s1.len().cmp(&s2.len()).then(s1.cmp(s2))
}

#[derive(PartialEq, Eq)]
enum Part<'a> {
    Num(&'a str),
    Str(&'a str),
}

fn partition_numbers(s: &str) -> impl Iterator<Item = Part<'_>> {
    let mut chars = s.chars().peekable();
    let mut pos = 0;
    iter::from_fn(move || {
        let beg = pos;
        let c = chars.next()?;
        let is_num = c.is_numeric();
        pos += c.len_utf8();
        while let Some(c) = chars.peek().copied() {
            if c.is_numeric() != is_num {
                break;
            }
            chars.next();
            pos += c.len_utf8();
        }
        let ss = &s[beg..pos];
        let p = match is_num {
            true => Part::Num(ss),
            false => Part::Str(ss),
        };
        Some(p)
    })
}

impl<'a> PartialOrd for Part<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for Part<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        use Part::*;
        match (self, other) {
            (Str(s1), Str(s2)) => natural_str(s1).cmp(natural_str(s2)),
            (Num(s1), Num(s2)) => cmp_num(s1, s2),
            (Str(s1), Num(s2)) => {
                natural_str(s1).next().cmp(&natural_str(s2).next())
            }
            (Num(s1), Str(s2)) => {
                natural_str(s1).next().cmp(&natural_str(s2).next())
            }
        }
    }
}

/// compare strings with awareness of number substrings
/// modeled after what Dolphin does
fn cmp_natural(a: &str, b: &str) -> cmp::Ordering {
    partition_numbers(a).cmp(partition_numbers(b))
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
        cmp_natural(&a, &b)
    });
    entries
}

/// read dir entries and sort them
pub fn dir_entries_sorted_rc(dir: &Path) -> Vec<Rc<PathBuf>> {
    let mut entries: Vec<_> = read_dir(dir).map(Rc::new).collect();
    entries.sort_by(|a, b| {
        let a = a.file_name().unwrap().to_string_lossy();
        let b = b.file_name().unwrap().to_string_lossy();
        cmp_natural(&a, &b)
    });
    entries
}

use egui_extras::loaders::webp_loader;

// hack to enable animations
pub fn animations_enable(ctx: &egui::Context) {
    if !ctx.is_loader_installed(webp_loader::WebPLoader::ID) {
        let webp_loader = webp_loader::WebPLoader::default();
        ctx.add_image_loader(std::sync::Arc::new(webp_loader));
    }
}

// hack to disable animations: remove the only loader that loads them (so far)
pub fn animations_disable(ctx: &egui::Context) {
    let loaders = ctx.loaders();
    let mut img_loaders = loaders.image.lock();
    if let Some(pos) = img_loaders
        .iter()
        .position(|loader| loader.id() == webp_loader::WebPLoader::ID)
    {
        img_loaders.remove(pos);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmp_natural() {
        assert_eq!(cmp_natural("_", "1"), Ordering::Less);
        assert_eq!(cmp_natural("1", "a"), Ordering::Less);
        assert_eq!(cmp_natural("a", "A"), Ordering::Less);
        assert_eq!(cmp_natural("A", "b"), Ordering::Less);
        assert_eq!(cmp_natural("1", "10"), Ordering::Less);
        assert_eq!(cmp_natural("2", "10"), Ordering::Less);
        assert_eq!(cmp_natural("1_1", "1_10"), Ordering::Less);
        assert_eq!(cmp_natural("1_10", "2_1"), Ordering::Less);
        assert_eq!(cmp_natural("ab123cd", "ab2345cd"), Ordering::Less);
    }
}
