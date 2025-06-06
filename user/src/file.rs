use super::*;
use alloc::string::String;

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }
}

pub fn dup(fd: usize) -> isize {
    sys_dup(fd)
}
pub fn open(path: &str, flags: OpenFlags) -> isize {
    sys_open(path, flags.bits)
}
pub fn close(fd: usize) -> isize {
    sys_close(fd)
}
pub fn pipe(pipe_fd: &mut [usize]) -> isize {
    sys_pipe(pipe_fd)
}
pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf)
}

pub fn getdents(path: &str) -> isize {
    sys_getdents(path)
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}

const NAME_LENGTH_LIMIT: usize = 27;
pub struct DirEntry {
    pub name: [u8; NAME_LENGTH_LIMIT + 1],
    pub inode_number: u32,
}

pub const DIRENT_SZ: usize = 32;

pub const CHR: usize = 0;
pub const REG: usize = 1;
pub const DIR: usize = 2;

pub struct Stat {
    pub ino: u32,
    pub mode: u32,
    pub off: u32,
    pub size: u32,
}

impl Stat {
    pub fn new() -> Self {
        Self {
            ino: 0,
            mode: 0,
            off: 0,
            size: 0,
        }
    }
}

pub fn mkdir(path: &str) -> isize {
    let path = String::from(path) + "\0";
    sys_mkdir(path.as_ptr())
}

pub fn fstat(fd: usize, stat: &mut Stat) -> isize {
    sys_fstat(fd, stat as *mut _ as *mut _)
}
