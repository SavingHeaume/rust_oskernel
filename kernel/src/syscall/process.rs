use crate::timer::get_time_us;
use alloc::sync::Arc;

use crate::{
    fs::{OpenFlags, open_file},
    mm::{translated_refmut, translated_str},
    task::{
        add_task, current_task, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next,
    },
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_task().unwrap().pid.0);
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    trace!("kernel:pid[{}] sys_fork", current_task().unwrap().pid.0);
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // 修改new_task的trap context，因为切换后会立即返回
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();

    trap_cx.x[10] = 0;

    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let task = current_task().unwrap();
        task.exec(all_data.as_slice());
        0
    } else {
        -1
    }
}

/// 如果没有与给定 pid 相同的子进程，则返回 -1。
/// 否则，如果存在子进程但仍在运行，则返回 -2。
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = current_task().unwrap();

    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        let exit_code = child.inner_exclusive_access().exit_code;
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
}

pub fn sys_get_time() -> isize {
    get_time_us() as isize
}

// pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
//     trace!(
//         "kernel:pid[{}] sys_mmap NOT IMPLEMENTED",
//         current_task().unwrap().pid.0
//     );
//     -1
// }

// pub fn sys_munmap(_start: usize, _len: usize) -> isize {
//     trace!(
//         "kernel:pid[{}] sys_munmap NOT IMPLEMENTED",
//         current_task().unwrap().pid.0
//     );
//     -1
// }

// pub fn sys_sbrk(size: i32) -> isize {
//     trace!("kernel:pid[{}] sys_sbrk", current_task().unwrap().pid.0);
//     if let Some(old_brk) = current_task().unwrap().change_program_brk(size) {
//         old_brk as isize
//     } else {
//         -1
//     }
// }

// pub fn sys_spawn(_path: *const u8) -> isize {
//     trace!(
//         "kernel:pid[{}] sys_spawn NOT IMPLEMENTED",
//         current_task().unwrap().pid.0
//     );
//     -1
// }

// pub fn sys_set_priority(_prio: isize) -> isize {
//     trace!(
//         "kernel:pid[{}] sys_set_priority NOT IMPLEMENTED",
//         current_task().unwrap().pid.0
//     );
//     -1
// }
