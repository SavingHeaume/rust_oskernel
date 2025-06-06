use super::File;
use crate::mm::UserBuffer;
use crate::sync::UPIntrFreeCell;
use crate::{
    drivers::BLOCK_DEVICE,
    fs::{DIR, LNK, REG},
};
use alloc::sync::Arc;
use alloc::vec::Vec;
use bitflags::*;
use file_system::{FileSystem, Inode};
use lazy_static::*;

pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: UPIntrFreeCell<OSInodeInner>,
}

pub struct OSInodeInner {
    offset: usize,
    inode: Arc<Inode>,
}

impl OSInode {
    pub fn new(readable: bool, writable: bool, inode: Arc<Inode>) -> Self {
        Self {
            readable,
            writable,
            inner: unsafe { UPIntrFreeCell::new(OSInodeInner { offset: 0, inode }) },
        }
    }
    pub fn read_all(&self) -> Vec<u8> {
        let mut inner = self.inner.exclusive_access();
        let mut buffer = [0u8; 512];
        let mut v: Vec<u8> = Vec::new();
        loop {
            let len = inner.inode.read_at(inner.offset, &mut buffer);
            if len == 0 {
                break;
            }
            inner.offset += len;
            v.extend_from_slice(&buffer[..len]);
        }
        v
    }
}

lazy_static! {
    pub static ref ROOT_INODE: Arc<Inode> = {
        let efs = FileSystem::open(BLOCK_DEVICE.clone());
        Arc::new(FileSystem::root_inode(&efs))
    };
}

pub fn find_inode(path: &str) -> Option<Arc<Inode>> {
    // println!("find_inode: {}", path);
    let root_inode = ROOT_INODE.clone();

    let componects: Vec<&str> = path.split('/').collect();
    // println!("split componects: {:?}", componects);

    componects.into_iter().fold(Some(root_inode), |res, name| {
        if let Some(node) = res {
            if !name.is_empty() {
                // println!("  ↳ Looking up file/dir: {:?}", name);
                node.find(name)
            } else {
                // println!("  ↳ Skipping empty component");
                Some(node)
            }
        } else {
            // println!("  ↳ Skipping (previous lookup failed)");
            None
        }
    })
}

/*
pub fn list_apps() {
    println!("/**** APPS ****");
    for app in ROOT_INODE.ls() {
        println!("{}", app);
    }
    println!("**************/
");
}
*/

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }
}

impl OpenFlags {
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}

pub fn open_file(name: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    // println!("open_file: {}", name);
    let (readable, writable) = flags.read_write();
    if flags.contains(OpenFlags::CREATE) {
        if let Some(inode) = find_inode(name) {
            // clear size
            inode.clear();
            Some(Arc::new(OSInode::new(readable, writable, inode)))
        } else {
            // create file
            let (parent_path, target) = name.rsplit_once('/').unwrap();
            let parent_inode = find_inode(parent_path).unwrap();
            parent_inode
                .create(target)
                .map(|inode| Arc::new(OSInode::new(readable, writable, inode)))
        }
    } else {
        find_inode(name).map(|inode| {
            if flags.contains(OpenFlags::TRUNC) {
                inode.clear();
            }
            Arc::new(OSInode::new(readable, writable, inode))
        })
    }
}

impl File for OSInode {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    fn read(&self, mut buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut total_read_size = 0usize;
        for slice in buf.buffers.iter_mut() {
            let read_size = inner.inode.read_at(inner.offset, *slice);
            if read_size == 0 {
                break;
            }
            inner.offset += read_size;
            total_read_size += read_size;
        }
        total_read_size
    }
    fn write(&self, buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut total_write_size = 0usize;
        for slice in buf.buffers.iter() {
            let write_size = inner.inode.write_at(inner.offset, *slice);
            assert_eq!(write_size, slice.len());
            inner.offset += write_size;
            total_write_size += write_size;
        }
        total_write_size
    }

    fn get_offset(&self) -> usize {
        self.inner.exclusive_access().offset
    }

    fn set_offset(&self, offset: usize) {
        self.inner.exclusive_access().offset = offset;
    }

    fn get_file_size(&self) -> usize {
        self.inner.exclusive_access().inode.get_file_size() as usize
    }

    fn get_inode_id(&self) -> usize {
        self.inner.exclusive_access().inode.get_inode_id() as usize
    }

    fn get_mode(&self) -> usize {
        let inode = &self.inner.exclusive_access().inode;
        if inode.is_file() {
            REG
        } else if inode.is_dir() {
            DIR
        } else {
            LNK
        }
    }
}
