#![no_std]
#![no_main]

use core::arch::global_asm;
use log::*;

#[macro_use]
mod console;
mod lang_items;
mod logging;
mod sbi;
mod sync;
mod trap;
mod syscall;
mod config;
mod task;
mod loader;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

pub fn clear_bss() {
    unsafe extern "C" {
        safe fn sbss();
        safe fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    logging::init();
    info!("hello, world!");
    trap::init();
    loader::load_apps();
    task::run_first_task();
    panic!("unreachable in rust_main!");
}