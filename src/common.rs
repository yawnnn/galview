#![allow(unused)]

use std::path::Path;

pub trait NumExt {
    fn modulo_add(self, rhs: Self, modulo: Self) -> Self;
    fn modulo_sub(self, rhs: Self, modulo: Self) -> Self;
}

impl NumExt for usize {
    fn modulo_add(self, rhs: Self, modulo: Self) -> Self {
        (self + rhs).rem_euclid(modulo)
    }

    fn modulo_sub(self, rhs: Self, modulo: Self) -> Self {
        (self as isize - rhs as isize).rem_euclid(modulo as isize) as usize
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
