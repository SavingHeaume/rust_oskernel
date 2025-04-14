mod inode;
mod stdio;

use crate::mm::UserBuffer;

pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}

/// inode的状态
#[repr(C)]
#[derive(Debug)]
pub struct Stat {
    /// 包含文件的设备ID
    pub dev: u64,
    /// 节点号
    pub ino: u64,
    /// file type and mode
    pub mode: StatMode,
    /// 硬链接数量
    pub nlink: u32,
    /// unused pad
    pad: [u64; 7],
}

bitflags! {
    /// The mode of a inode
    /// directory or file
    pub struct StatMode: u32 {
        const NULL  = 0;
        const DIR   = 0o040000;
        const FILE  = 0o100000;
    }
}

pub use inode::{OSInode, OpenFlags, list_apps, open_file};
pub use stdio::{Stdin, Stdout};
