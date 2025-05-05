#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
extern crate bitflags;

use drivers::{KEYBOARD_DEVICE, MOUSE_DEVICE, gpu::GPU_DEVICE};
use log::*;

#[macro_use]
mod console;
mod config;
mod drivers;
mod fs;
mod lang_items;
mod logging;
mod mm;
mod sbi;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;

use crate::drivers::chardev::{CharDevice, UART};
use core::arch::global_asm;
use lazy_static::lazy_static;
use sync::UPIntrFreeCell;

global_asm!(include_str!("entry.asm"));

fn clear_bss() {
    unsafe extern "C" {
        safe fn sbss();
        safe fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

lazy_static! {
    pub static ref DEV_NON_BLOCKING_ACCESS: UPIntrFreeCell<bool> =
        unsafe { UPIntrFreeCell::new(false) };
}

/// the rust entry-point of os
#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    logging::init();
    mm::init();
    UART.init();
    info!("[kernel] Hello, world!");
    // mm::remap_test();

    info!("[kernel] gpu init");
    let _gpe = GPU_DEVICE.clone();
    info!("[kernel] keyboard init");
    let _keyboard = KEYBOARD_DEVICE.clone();
    info!("[kernel] mouse init");
    let _mouse = MOUSE_DEVICE.clone();

    info!("[kernel] trap init");
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();

    config::device_init();

    task::add_initproc();
    *DEV_NON_BLOCKING_ACCESS.exclusive_access() = true;
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
