#![allow(unused)]

use std::path::Path;

pub trait NumExt {
    type Signed;
    fn saturating_add_on(self, rhs: Self::Signed, on: Self) -> Self;
    fn saturating_add_on_or_stay(self, rhs: Self::Signed, on: Self) -> Self;
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
