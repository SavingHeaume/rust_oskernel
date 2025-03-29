use core::cell::RefMut;

use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;

use super::ProcessContext;
use super::pid::{KernelStack, PidHandle};
use crate::config::{TRAP_CONTEXT, kernel_stack_position};
use crate::mm::{KERNEL_SPACE, MapPermission, MemorySet};
use crate::mm::{PhysPageNum, VirtAddr};
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use crate::trap::trap_handler;

#[derive(Copy, Clone, PartialEq)]
pub enum ProcessStatus {
    Ready,
    Running,
    Zombie,
}

pub struct ProcessControlBlock {
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    inner: UPSafeCell<ProcessControlBlockInner>,
}

pub struct ProcessControlBlockInner {
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
    pub process_status: ProcessStatus,
    pub process_cx: ProcessContext,
    pub memory_set: MemorySet,

    pub parent: Option<Weak<ProcessControlBlock>>,
    pub children: Vec<Arc<ProcessControlBlock>>,

    pub exit_code: i32,
}

impl ProcessControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }

    pub fn get_status(&self) -> ProcessStatus {
        self.process_status
    }

    pub fn is_zombie(&self) -> bool {
        self.get_status() == ProcessStatus::Zombie
    }
}

impl ProcessControlBlock {
    pub fn inner_get_mut(&self) -> RefMut<'_, ProcessControlBlockInner> {
        self.inner.exclusive_access()
    }

    pub fn getpid(&self) -> usize {
        self.pid.0
    }

    pub fn new(elf_data: &[u8]) -> Self {}
    pub fn exec(&self, elf_data: &[u8]) {}
    pub fn fork(self: &Arc<ProcessControlBlock>) -> Arc<ProcessControlBlock> {}
}
