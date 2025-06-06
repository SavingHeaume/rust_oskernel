mod inode;
mod pipe;
mod stdio;

use crate::mm::UserBuffer;
use alloc::sync::Arc;

const CHR: usize = 0;
const REG: usize = 1;
const DIR: usize = 2;
const LNK: usize = 3;

const EOT: char = '\x04';
const LF: char = '\x0a';
const CR: char = '\x0d';

pub struct Stat {
    pub ino: u32,
    pub mode: u32,
    pub off: u32,
    pub size: u32,
}

impl From<Arc<dyn File + Send + Sync>> for Stat {
    fn from(file: Arc<dyn File + Send + Sync>) -> Self {
        Self {
            ino: file.get_inode_id() as u32,
            mode: file.get_mode() as u32,
            off: file.get_offset() as u32,
            size: file.get_file_size() as u32,
        }
    }
}

pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;

    fn get_offset(&self) -> usize {
        0
    }
    fn set_offset(&self, _offset: usize) {}
    fn get_file_size(&self) -> usize {
        0
    }
    fn get_inode_id(&self) -> usize {
        0
    }
    fn get_mode(&self) -> usize {
        CHR
    }
}

pub use inode::{OpenFlags, ROOT_INODE, find_inode, open_file};
pub use pipe::make_pipe;
pub use stdio::{Stdin, Stdout};
