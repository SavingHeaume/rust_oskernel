#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate log;

use core::arch::global_asm;
use log::*;

#[macro_use]
mod console;
mod config;
mod drivers;
mod fs;
mod lang_items;
mod logging;
mod mm;
mod process;
mod sbi;
mod sync;
mod syscall;
mod timer;
mod trap;

global_asm!(include_str!("entry.asm"));

pub fn clear_bss() {
    unsafe extern "C" {
        safe fn sbss();
        safe fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0)
    }
}

#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    logging::init();
    info!("hello, world!");
    mm::init();
    mm::remap_test();
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    fs::list_apps();
    process::add_initproc();
    process::run_process();
    panic!("unreachable in rust_main!");
}
