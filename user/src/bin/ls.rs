#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::getdents;

#[unsafe(no_mangle)]
pub fn main(argc: usize, argv: &[&str]) -> i32 {
    if argc == 2 {
        getdents(argv[1]);
    } else {
        println!("wrong parameter");
    }
    0
}
