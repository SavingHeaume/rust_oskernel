mod context;
mod manager;
mod pid;
mod process;
mod processor;
mod switch;

use crate::loader::get_app_data_by_name;
use alloc::sync::Arc;
use context::ProcessContext;
use lazy_static::lazy_static;
use manager::add_process;
use process::{ProcessControlBlock, ProcessStatus};
use processor::{schedule, take_current_process};
pub use processor::current_user_token;

lazy_static! {
    pub static ref INITPROC: Arc<ProcessControlBlock> = Arc::new(ProcessControlBlock::new(
        get_app_data_by_name("initproc").unwrap()
    ));
}

pub fn add_initproc() {
    add_process(INITPROC.clone());
}

pub fn suspend_current_and_run_next() {
    let process = take_current_process().unwrap();

    let mut process_inner = process.inner_get_mut();
    let process_cx_ptr = &mut process_inner.process_cx as *mut ProcessContext;

    process_inner.process_status = ProcessStatus::Ready;
    drop(process_inner);

    add_process(process);
    schedule(process_cx_ptr);
}
