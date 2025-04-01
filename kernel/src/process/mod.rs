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
pub use manager::add_process;
use process::{ProcessControlBlock, ProcessStatus};
pub use processor::{current_process, current_trap_cx, current_user_token, run_process};
use processor::{schedule, take_current_process};

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

pub fn exit_current_and_run_next(exit_code: i32) {
    let process = take_current_process().unwrap();
    let mut inner = process.inner_get_mut();
    inner.process_status = ProcessStatus::Zombie;
    inner.exit_code = exit_code;

    {
        let mut initproc_inner = INITPROC.inner_get_mut();
        for child in inner.children.iter() {
            child.inner_get_mut().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }

    inner.children.clear();
    inner.memory_set.recycle_data_pages();
    drop(inner);
    drop(process);
    let mut _unused = ProcessContext::zero_init();
    schedule(&mut _unused as *mut _);
}
