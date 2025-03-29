use crate::{sync::UPSafeCell, trap::TrapContext};

use super::{context::ProcessContext, process::ProcessControlBlock};
use alloc::sync::Arc;
use lazy_static::lazy_static;

pub struct Processor {
    current: Option<Arc<ProcessControlBlock>>,
    idle_task_cx: ProcessContext,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: ProcessContext::zero_init(),
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
}

pub fn current_process() -> Option<Arc<ProcessControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

pub fn current_user_token() -> usize {
    let process = current_process().unwrap();
    let token = process.inner_get_mut().get_user_token();
    token
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    current_process().unwrap().inner_get_mut().get_trap_cx()
}
