#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{OpenFlags, close, open};

#[unsafe(no_mangle)]
pub fn main(argc: usize, argv: &[&str]) -> i32 {
    assert!(argc == 2);
    let fd = open(argv[1], OpenFlags::CREATE);
    if fd == -1 {
        panic!("error when crate file")
    }
    close(fd as usize);
    println!("create file: {}", argv[1]);
    0
}
