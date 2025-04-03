#![no_std]
#![no_main]

use user_lib::{OpenFlags, close, open, read, write};

#[macro_use]
extern crate user_lib;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    let test_str = "hello, world!";
    let filea = "filea\0";
    let fd = open(filea, OpenFlags::CREATE | OpenFlags::WRONLY);
    assert!(fd > 0);
    let fd = fd as usize;
    write(fd, test_str.as_bytes());
    close(fd);

    let fd = open(filea, OpenFlags::RDONLY);
    let fd = fd as usize;
    let mut buffer = [0u8; 100];
    let read_len = read(fd, &mut buffer) as usize;
    close(fd);
    println!("test_str: {}", test_str);
    println!(
        "read from file: {}",
        core::str::from_utf8(&buffer[..read_len]).unwrap()
    );
    0
}
