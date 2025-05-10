#![no_std]
#![no_main]

extern crate user_lib;

use user_lib::getdents;

#[unsafe(no_mangle)]
pub fn main(_argc: usize, _argv: &[&str]) -> i32 {
    getdents(0, &mut [0; 1]);
    0
}
