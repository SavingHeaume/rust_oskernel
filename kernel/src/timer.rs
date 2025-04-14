use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;
use riscv::register::time;
/// The number of ticks per second
const TICKS_PER_SEC: usize = 100;
/// The number of milliseconds per second
const MSEC_PER_SEC: usize = 1000;
/// The number of microseconds per second
const MICRO_PER_SEC: usize = 1_000_000;

pub fn get_time() -> usize {
    time::read()
}

pub fn get_time_ms() -> usize {
    time::read() * MSEC_PER_SEC / CLOCK_FREQ
}


pub fn get_time_us() -> usize {
    time::read() * MICRO_PER_SEC / CLOCK_FREQ
}

pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}
