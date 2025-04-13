use crate::{sync::UPSafeCell, trap::TrapContext};
use log::warn;

use super::{
    context::ProcessContext,
    manager::fetch_process,
    process::{ProcessControlBlock, ProcessStatus},
    switch::__switch,
};
use alloc::sync::Arc;
use lazy_static::lazy_static;

pub struct Processor {
    current: Option<Arc<ProcessControlBlock>>,
    idle_process_cx: ProcessContext,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            current: None,
            idle_process_cx: ProcessContext::zero_init(),
        }
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}

impl Processor {
    // 取出当前正在执行的任务。
    pub fn take_current(&mut self) -> Option<Arc<ProcessControlBlock>> {
        self.current.take()
    }

    // 返回当前执行的任务的一份拷贝。
    pub fn current(&self) -> Option<Arc<ProcessControlBlock>> {
        self.current.as_ref().map(|process| Arc::clone(process))
    }

    fn get_idle_process_cx_ptr(&mut self) -> *mut ProcessContext {
        &mut self.idle_process_cx as *mut _
    }
}

pub fn current_process() -> Option<Arc<ProcessControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

pub fn take_current_process() -> Option<Arc<ProcessControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

pub fn current_user_token() -> usize {
    let process = current_process().unwrap();
    let token = process.inner_get_mut().get_user_token();
    token
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    current_process().unwrap().inner_get_mut().get_trap_cx()
}
// idle 控制流，它运行在这个 CPU 核的启动栈上，
// 功能是尝试从任务管理器中选出一个任务来在当前 CPU 核上执行。
// 在内核初始化完毕之后，会通过调用 run_tasks 函数来进入 idle 控制流：
pub fn run_process() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(process) = fetch_process() {
            let idle_process_cx_ptr = processor.get_idle_process_cx_ptr();
            let mut process_innier = process.inner_get_mut();
            let next_process_cx_ptr = &process_innier.process_cx as *const ProcessContext;
            process_innier.process_status = ProcessStatus::Running;

            drop(process_innier);
            processor.current = Some(process);
            drop(processor);

            unsafe {
                __switch(idle_process_cx_ptr, next_process_cx_ptr);
            }
        } else {
            warn!("no process available in run_process");
        }
    }
}

// 当一个应用用尽了内核本轮分配给它的时间片或者它主动调用 yield 系统调用交出 CPU 使用权之后
// 内核会调用 schedule 函数来切换到 idle 控制流并开启新一轮的任务调度。
pub fn schedule(switched_process_cx_ptr: *mut ProcessContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_process_cx_ptr = processor.get_idle_process_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_process_cx_ptr, idle_process_cx_ptr);
    }
}
