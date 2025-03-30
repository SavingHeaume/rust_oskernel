use core::arch::global_asm;
use super::ProcessContext;

global_asm!(include_str!("switch.S"));

unsafe extern "C" {
    pub fn __switch(
        current_process_cx_ptr: *mut ProcessContext,
        next_process_cx_ptr: *const ProcessContext,
    );
}