use super::TaskContext;
use super::{KernelStack, PidHandle, kstack_alloc, pid_alloc};
use crate::config::TRAP_CONTEXT_BASE;
use crate::fs::{File, Stdin, Stdout};
use crate::mm::{KERNEL_SPACE, MemorySet, PhysPageNum, VirtAddr};
use crate::sync::UPSafeCell;
use crate::trap::{TrapContext, trap_handler};
use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefMut;


pub struct TaskControlBlock {

    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    inner: UPSafeCell<TaskControlBlockInner>,
}

impl TaskControlBlock {
    pub fn inner_exclusive_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }

    pub fn get_user_token(&self) -> usize {
        let inner = self.inner_exclusive_access();
        inner.memory_set.token()
    }
}

pub struct TaskControlBlockInner {
    /// 放置陷阱上下文的页帧的物理页号
    pub trap_cx_ppn: PhysPageNum,

    /// 应用程序数据只能出现在应用程序地址空间小于 base_size 的区域
    pub base_size: usize,

    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,
    pub parent: Option<Weak<TaskControlBlock>>,

    pub children: Vec<Arc<TaskControlBlock>>,
    pub exit_code: i32,
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,

    pub heap_bottom: usize,
    pub program_brk: usize,
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
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

impl TaskControlBlock {
    /// 创建新进程
    ///
    /// 目前仅用于创建 initproc
    pub fn new(elf_data: &[u8]) -> Self {
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT_BASE).into())
            .unwrap()
            .ppn();

        // 在内核空间中分配一个 pid 和一个内核堆栈
        let pid_handle = pid_alloc();
        let kernel_stack = kstack_alloc();
        let kernel_stack_top = kernel_stack.get_top();
        // 将进入 trap_return 的任务上下文推送到内核堆栈顶部
        let task_control_block = Self {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx_ppn,
                    base_size: user_sp,
                    task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    memory_set,
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: vec![
                        // 0 -> stdin
                        Some(Arc::new(Stdin)),
                        // 1 -> stdout
                        Some(Arc::new(Stdout)),
                        // 2 -> stderr
                        Some(Arc::new(Stdout)),
                    ],
                    heap_bottom: user_sp,
                    program_brk: user_sp,
                })
            },
        };
        // 在用户空间准备 TrapContext
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }

    /// 加载新的 elf 替换原有的应用程序地址空间并开始执行
    pub fn exec(&self, elf_data: &[u8]) {

        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT_BASE).into())
            .unwrap()
            .ppn();

        let mut inner = self.inner_exclusive_access();
        // 替代memory_set
        inner.memory_set = memory_set;
        // 更新 trap_cx ppn
        inner.trap_cx_ppn = trap_cx_ppn;
        // 初始化trap_cx
        let trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            self.kernel_stack.get_top(),
            trap_handler as usize,
        );
        *inner.get_trap_cx() = trap_cx;
    }

    /// 父进程 fork 子进程
    pub fn fork(self: &Arc<TaskControlBlock>) -> Arc<TaskControlBlock> {
        let mut parent_inner = self.inner_exclusive_access();
        // 复制用户空间，包括trap上下文
        let memory_set = MemorySet::from_existed_user(&parent_inner.memory_set);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT_BASE).into())
            .unwrap()
            .ppn();

        // 在内核空间中分配一个 pid 和一个内核堆栈
        let pid_handle = pid_alloc();
        let kernel_stack = kstack_alloc();
        let kernel_stack_top = kernel_stack.get_top();
        // 复制 fd 表
        let mut new_fd_table: Vec<Option<Arc<dyn File + Send + Sync>>> = Vec::new();
        for fd in parent_inner.fd_table.iter() {
            if let Some(file) = fd {
                new_fd_table.push(Some(file.clone()));
            } else {
                new_fd_table.push(None);
            }
        }
        let task_control_block = Arc::new(TaskControlBlock {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx_ppn,
                    base_size: parent_inner.base_size,
                    task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    memory_set,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: new_fd_table,
                    heap_bottom: parent_inner.heap_bottom,
                    program_brk: parent_inner.program_brk,
                })
            },
        });

        parent_inner.children.push(task_control_block.clone());
        // 修改 trap_cx 中的 kernel_sp
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;

        task_control_block
    }

    pub fn getpid(&self) -> usize {
        self.pid.0
    }

    /// 改变程序中断的位置。如果失败则返回 None。
    pub fn change_program_brk(&self, size: i32) -> Option<usize> {
        let mut inner = self.inner_exclusive_access();
        let heap_bottom = inner.heap_bottom;
        let old_break = inner.program_brk;
        let new_brk = inner.program_brk as isize + size as isize;
        if new_brk < heap_bottom as isize {
            return None;
        }
        let result = if size < 0 {
            inner
                .memory_set
                .shrink_to(VirtAddr(heap_bottom), VirtAddr(new_brk as usize))
        } else {
            inner
                .memory_set
                .append_to(VirtAddr(heap_bottom), VirtAddr(new_brk as usize))
        };
        if result {
            inner.program_brk = new_brk as usize;
            Some(old_break)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
/// task status: UnInit, Ready, Running, Exited
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Zombie,
}
