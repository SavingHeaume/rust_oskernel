#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{close, console::print, open, read, write, OpenFlags};

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    let test_str = "Hello, world!";
    let filea = "filea\0";
    println!("test_str: {}", &test_str);
    println!("file: {}", &filea);
    println!("open");
    let fd = open(filea, OpenFlags::CREATE | OpenFlags::WRONLY);
    assert!(fd > 0);
    let fd = fd as usize;
    println!("write");
    write(fd, test_str.as_bytes());
    println!("close");
    close(fd);

    println!("open");
    let fd = open(filea, OpenFlags::RDONLY);
    assert!(fd > 0);
    let fd = fd as usize;
    let mut buffer = [0u8; 100];
    println!("read");
    let read_len = read(fd, &mut buffer) as usize;
    println!("close");
    close(fd);

    assert_eq!(test_str, core::str::from_utf8(&buffer[..read_len]).unwrap(),);
    println!("file_test passed!");
    0
}
