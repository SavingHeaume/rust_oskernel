use crate::sync::UPSafeCell;
use alloc::{collections::vec_deque::VecDeque, sync::Arc};

use super::process::ProcessControlBlock;
use lazy_static::lazy_static;

pub struct ProcessManager {
    ready_queue: VecDeque<Arc<ProcessControlBlock>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }

    pub fn add(&mut self, process: Arc<ProcessControlBlock>) {
        self.ready_queue.push_back(process);
    }

    pub fn fetch(&mut self) -> Option<Arc<ProcessControlBlock>> {
        self.ready_queue.pop_front()
    }
}

lazy_static! {
    pub static ref PROCESS_MANAGER: UPSafeCell<ProcessManager> =
        unsafe { UPSafeCell::new(ProcessManager::new()) };
}

pub fn add_process(process: Arc<ProcessControlBlock>) {
    PROCESS_MANAGER.exclusive_access().add(process);
}

pub fn fetch_process() -> Option<Arc<ProcessControlBlock>> {
    PROCESS_MANAGER.exclusive_access().fetch()
}
