#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use core::str;
use user_lib::*;

#[unsafe(no_mangle)]
pub fn main(argc: usize, argv: &[&str]) -> i32 {
    if argc == 1 {
        println!("missing operand");
        return 1;
    }
    for target in &argv[1..] {
        match unlink(target, AT_REMOVEDIR) {
            0 => println!("remove success"),
            -1 => println!("failed to remove '{}': No such file or directory", target),
            -2 => println!("failed to remove '{}': Not a directory", target),
            -3 => println!("failed to remove '{}': Directory not empty", target),
            _ => panic!(),
        }
    }
    0
}
