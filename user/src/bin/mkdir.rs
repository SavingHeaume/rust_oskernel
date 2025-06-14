#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use core::str;
use user_lib::*;

#[unsafe(no_mangle)]
fn main(argc: usize, argv: &[&str]) -> i32 {
    if argc == 1 {
        println!("missing operand");
        return 1;
    }

    for target in &argv[1..] {
        match mkdir(target) {
            0 => println!("create success"),
            -1 => println!(
                "connot create directory {}: No such file or directory",
                target
            ),
            -2 => println!("cannot create directory {}: File exists", target),
            _ => panic!(),
        }
    }
    0
}
