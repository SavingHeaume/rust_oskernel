#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{OpenFlags, close, getdents, open};

#[unsafe(no_mangle)]
pub fn main(argc: usize, argv: &[&str]) -> i32 {
    getdents(0, &mut [0; 1]);
    0
}
