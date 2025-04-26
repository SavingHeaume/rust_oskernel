// fs
const SYSCALL_DUP: usize = 24;
const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_PIPE: usize = 59;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;

// process
const SYSCALL_EXIT: usize = 93;
const SYSCALL_SLEEP: usize = 101;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_KILL: usize = 129;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;

// thread
const SYSCALL_THREAD_CREATE: usize = 1000;
const SYSCALL_GETTID: usize = 1001;
const SYSCALL_WAITTID: usize = 1002;

// mutex
const SYSCALL_MUTEX_CREATE: usize = 1010;
const SYSCALL_MUTEX_LOCK: usize = 1011;
const SYSCALL_MUTEX_UNLOCK: usize = 1012;

const SYSCALL_GETDENTS: usize = 61;

mod fs;
mod process;
mod sync;
mod thread;

use fs::*;
use process::*;
use sync::*;
use thread::*;

use log::*;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_DUP => {
            info!("syscall_dup");
            sys_dup(args[0])
        }
        SYSCALL_OPEN => {
            info!("syscall_open");
            sys_open(args[0] as *const u8, args[1] as u32)
        }
        SYSCALL_CLOSE => {
            info!("syscall_close");
            sys_close(args[0])
        }
        SYSCALL_PIPE => {
            info!("syscall_pipe");
            sys_pipe(args[0] as *mut usize)
        }
        SYSCALL_READ => sys_read(args[0], args[1] as *const u8, args[2]),
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_SLEEP => {
            info!("syscall_sleep");
            sys_sleep(args[0])
        }
        SYSCALL_EXIT => {
            info!("syscall_exit");
            sys_exit(args[0] as i32)
        }
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_KILL => {
            info!("syscall_kill");
            sys_kill(args[0], args[1] as i32)
        }

        SYSCALL_GET_TIME => sys_get_time(),
        SYSCALL_GETPID => {
            info!("syscall_gitpid");
            sys_getpid()
        }
        SYSCALL_FORK => {
            info!("syscall_fork");
            sys_fork()
        }
        SYSCALL_EXEC => {
            info!("syscall_exec");
            sys_exec(args[0] as *const u8, args[1] as *const usize)
        }
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),
        SYSCALL_THREAD_CREATE => {
            info!("syscall_thread_create");
            sys_thread_create(args[0], args[1])
        }
        SYSCALL_WAITTID => {
            info!("syscall_waittid");
            sys_waittid(args[0]) as isize
        }
        SYSCALL_MUTEX_CREATE => {
            info!("syscall_mutex_create");
            sys_mutex_create(args[0] == 1)
        }
        SYSCALL_MUTEX_LOCK => {
            info!("syscall_mutex_lock");
            sys_mutex_lock(args[0])
        }
        SYSCALL_MUTEX_UNLOCK => {
            info!("syscall_mutex_unlock");
            sys_mutex_unlock(args[0])
        }
        SYSCALL_GETDENTS => {
            info!("syscall_getdents");
            sys_getdents(args[0], args[1] as *const u8, args[2])
        }
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
