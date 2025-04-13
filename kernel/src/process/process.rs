use core::cell::RefMut;

use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;

use super::ProcessContext;
use super::pid::pid_alloc;
use super::pid::{KernelStack, PidHandle};
use crate::config::TRAP_CONTEXT;
use crate::fs::File;
use crate::fs::{Stdin, Stdout};
use crate::mm::{KERNEL_SPACE, MemorySet};
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
    pub process_cx: ProcessContext,
    pub process_status: ProcessStatus,
    pub memory_set: MemorySet,

    pub parent: Option<Weak<ProcessControlBlock>>,
    pub children: Vec<Arc<ProcessControlBlock>>,

    pub exit_code: i32,

    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,
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

    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }
}

impl ProcessControlBlock {
    pub fn inner_get_mut(&self) -> RefMut<'_, ProcessControlBlockInner> {
        self.inner.exclusive_access()
    }

    pub fn getpid(&self) -> usize {
        self.pid.0
    }

    pub fn new(elf_data: &[u8]) -> Self {
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();

        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();

        let process_control_block = Self {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(ProcessControlBlockInner {
                    trap_cx_ppn,
                    base_size: user_sp,
                    process_cx: ProcessContext::goto_trap_return(kernel_stack_top),
                    process_status: ProcessStatus::Ready,
                    memory_set,
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: vec![
                        Some(Arc::new(Stdin)),
                        Some(Arc::new(Stdout)),
                        Some(Arc::new(Stdout)),
                    ],
                })
            },
        };

        let trap_cx = process_control_block.inner_get_mut().get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );

        process_control_block
    }
    pub fn fork(self: &Arc<ProcessControlBlock>) -> Arc<ProcessControlBlock> {
        let mut parent_inner = self.inner_get_mut();
        let memory_set = MemorySet::from_existed_user(&parent_inner.memory_set);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();

        let mut new_fd_table: Vec<Option<Arc<dyn File + Send + Sync>>> = Vec::new();
        for fd in parent_inner.fd_table.iter() {
            if let Some(file) = fd {
                new_fd_table.push(Some(file.clone()));
            } else {
                new_fd_table.push(None);
            }
        }

        let pcb = Arc::new(ProcessControlBlock {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(ProcessControlBlockInner {
                    trap_cx_ppn,
                    base_size: parent_inner.base_size,
                    process_cx: ProcessContext::goto_trap_return(kernel_stack_top),
                    process_status: ProcessStatus::Ready,
                    memory_set,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: new_fd_table,
                })
            },
        });
        parent_inner.children.push(pcb.clone());
        let trap_cx = pcb.inner_get_mut().get_trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;

        pcb
    }
    pub fn exec(&self, elf_data: &[u8]) {
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();

        let mut inner = self.inner_get_mut();
        inner.memory_set = memory_set;
        inner.trap_cx_ppn = trap_cx_ppn;
        let trap_cx = inner.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            self.kernel_stack.get_top(),
            trap_handler as usize,
        );
    }
}
