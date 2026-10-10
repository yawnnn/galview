#![allow(unused)]

use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

thread_local! {
    static FRAME_COUNT: Cell<u64> = const { Cell::new(0)};
}

/// Call when a frame starts
pub fn frame_start() {
    FRAME_COUNT.with(|frame_count| {
        frame_count.update(|v| v.wrapping_add(1));
    })
}

pub fn frame_count() -> u64 {
    FRAME_COUNT.get()
}

type SrcPosition = (&'static str, u32, u32);

#[macro_export]
macro_rules! src_pos {
    () => {
        (file!(), line!(), column!())
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

pub fn run_one_frame(src_pos: SrcPosition, not_first: bool, inner: impl FnOnce()) {
    let frame_count = frame_count();
    if not_first && frame_count <= 1 {
        return;
    }

    RAN_X_FRAME.with_borrow_mut(|ran| match ran.get(&src_pos) {
        Some(v) if *v == frame_count => inner(),
        None => {
            inner();
            ran.insert(src_pos, frame_count);
        }
        _ => (),
    });
}

thread_local! {
    static RAN_X_TIME: RefCell<HashMap<SrcPosition, Instant>> = RefCell::default();
}

pub fn run_timed(src_pos: SrcPosition, timer: Duration, inner: impl FnOnce()) {
    RAN_X_TIME.with_borrow_mut(|ran| match ran.get(&src_pos) {
        Some(v) if v.elapsed() >= timer => {
            inner();
            ran.insert(src_pos, Instant::now());
        }
        None => {
            inner();
            ran.insert(src_pos, Instant::now());
        }
        _ => (),
    });
}

#[macro_export]
macro_rules! print_once {
    ($($arg:tt)*) => {{
        $crate::logging::run_once($crate::src_pos!(), || {
            print!($($arg)*)
        });
    }};
}

#[macro_export]
macro_rules! println_once {
    ($($arg:tt)*) => {{
        $crate::logging::run_once($crate::src_pos!(), || {
            println!($($arg)*)
        });
    }};
}

#[macro_export]
macro_rules! print_one_frame {
    ($not_first:expr, $($arg:tt)*) => {{
        $crate::logging::run_one_frame($crate::src_pos!(), $not_first, || {
            print!($($arg)*)
        });
    }};
}

#[macro_export]
macro_rules! println_one_frame {
    ($not_first:expr, $($arg:tt)*) => {{
        $crate::logging::run_one_frame($crate::src_pos!(), $not_first, || {
            println!($($arg)*)
        });
    }};
}

#[macro_export]
macro_rules! print_timed {
    ($timer:expr, $($arg:tt)*) => {{
        $crate::logging::run_timed($crate::src_pos!(), std::time::Duration::from_millis($timer), || {
            print!($($arg)*)
        });
    }};
}

#[macro_export]
macro_rules! println_timed {
    ($timer:expr, $($arg:tt)*) => {{
        $crate::logging::run_timed($crate::src_pos!(), std::time::Duration::from_millis($timer), || {
            println!($($arg)*)
        });
    }};
}