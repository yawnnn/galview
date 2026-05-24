#![allow(unused)]

use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

type SrcPosition = (*const u8, u32, u32);

thread_local! {
    static FRAME_COUNT: Cell<u64> = const { Cell::new(0)};
}

pub fn new_frame() {
    FRAME_COUNT.with(|frame_count| {
        frame_count.update(|v| v.wrapping_add(1));
    })
}

pub fn current_frame() -> u64 {
    FRAME_COUNT.get()
}

#[macro_export]
macro_rules! srcpos {
    () => {
        (file!().as_ptr(), line!(), column!())
    };
}

thread_local! {
    static RAN_X_SRC_POS: RefCell<HashSet<SrcPosition>> = RefCell::default();
}

pub fn run_once(src_pos: SrcPosition, inner: impl FnOnce()) {
    RAN_X_SRC_POS.with_borrow_mut(|ran| {
        if !ran.contains(&src_pos) {
            inner();
            ran.insert(src_pos);
        }
    });
}

thread_local! {
    static RAN_X_FRAME: RefCell<HashMap<SrcPosition, u64>> = RefCell::default();
}

pub fn run_one_frame(src_pos: SrcPosition, inner: impl FnOnce()) {
    RAN_X_FRAME.with_borrow_mut(|ran| match ran.get(&src_pos) {
        Some(v) if *v == current_frame() => inner(),
        None => {
            inner();
            ran.insert(src_pos, current_frame());
        }
        _ => (),
    });
}

#[macro_export]
macro_rules! println_once {
    ($($arg:tt)*) => {{
        $crate::common::run_once(srcpos!(), || {
            println!($($arg)*)
        });
    }};
}

#[macro_export]
macro_rules! println_one_frame {
    ($($arg:tt)*) => {{
        $crate::common::run_one_frame(srcpos!(), || {
            println!($($arg)*)
        });
    }};
}

pub fn resource(name: impl AsRef<Path>) -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("resources")
        .join(name)
}
