use crate::fs::{OpenFlags, open_file};
use crate::mm::{translated_refmut, translated_str};
use crate::process::{
    add_process, current_process, current_user_token, exit_current_and_run_next,
    suspend_current_and_run_next,
};
use crate::timer::get_time_us;
use alloc::sync::Arc;
use log::trace;

pub fn sys_exit(exit_code: i32) -> ! {
    exit_current_and_run_next(exit_code);
    panic!("Unreadchable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time() -> isize {
    get_time_us() as isize
}

pub fn sys_fork() -> isize {
    let current_process = current_process().unwrap();
    let new_process = current_process.fork();
    let new_pid = new_process.pid.0;

    let trap_cx = new_process.inner_get_mut().get_trap_cx();
    trap_cx.x[10] = 0;
    add_process(new_process);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_process().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let process = current_process().unwrap();
        process.exec(all_data.as_slice());
        0
    } else {
        -1
    }
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let process = current_process().unwrap();
    let mut inner = process.inner_get_mut();
    if inner
        .children
        .iter()
        .find(|p| pid == -1 || pid as usize == p.getpid())
        .is_none()
    {
        return -1;
    }

    let pair =
        inner.children.iter().enumerate().find(|(_, p)| {
            p.inner_get_mut().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        });

    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        let exit_code = child.inner_get_mut().exit_code;
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
}

pub fn sys_getpid() -> isize {
    current_process().unwrap().pid.0 as isize
}
