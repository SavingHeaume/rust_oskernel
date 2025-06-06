use crate::fs::{OpenFlags, ROOT_INODE, Stat, find_inode, make_pipe, open_file};
use crate::mm::{UserBuffer, translated_byte_buffer, translated_refmut, translated_str};
use crate::task::{current_process, current_user_token};
use alloc::sync::Arc;
use core::ptr::slice_from_raw_parts;

pub fn sys_getdents(path: *const u8) -> isize {
    let path = translated_str(current_user_token(), path);
    let inode = find_inode(path.as_str());
    let vec = inode.unwrap().ls();
    let max_width = vec.iter().map(|s| s.len()).max().unwrap_or(0);

    for chunk in vec.chunks(5) {
        for s in chunk {
            print!("{:width$}  ", s, width = max_width); // 两个空格分隔
        }
        print!("\n");
    }
    0
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let process = current_process();
    let inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let process = current_process();
    let inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        drop(inner);
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let process = current_process();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = process.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let process = current_process();
    let token = current_user_token();
    let mut inner = process.inner_exclusive_access();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = inner.alloc_fd();
    inner.fd_table[read_fd] = Some(pipe_read);
    let write_fd = inner.alloc_fd();
    inner.fd_table[write_fd] = Some(pipe_write);
    // 将读端和写端的文件描述符写回到应用地址空间
    *translated_refmut(token, pipe) = read_fd;
    *translated_refmut(token, unsafe { pipe.add(1) }) = write_fd;
    0
}

pub fn sys_dup(fd: usize) -> isize {
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    let new_fd = inner.alloc_fd();
    inner.fd_table[new_fd] = Some(Arc::clone(inner.fd_table[fd].as_ref().unwrap()));
    new_fd as isize
}

pub fn sys_mkdir(path: *const u8) -> isize {
    let token = current_user_token();
    let dir = translated_str(token, path);
    let (parent_path, target) = dir.rsplit_once('/').unwrap();

    if let Some(parent_inode) = find_inode(parent_path) {
        if let Some(_cur_inode) = parent_inode.create_dir(target) {
            0
        } else {
            -2
        }
    } else {
        -1
    }
}

pub fn sys_fstat(fd: usize, stat: *mut u8) -> isize {
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    let token = current_user_token();
    let user_buffer = UserBuffer::new(translated_byte_buffer(token, stat, size_of::<Stat>()));

    let fd_table = &mut inner.fd_table;

    if fd >= fd_table.len() || fd_table[fd].is_none() {
        return -1;
    }

    let file = fd_table[fd].clone().unwrap();
    let tmp_stat = Stat::from(file);
    let stat_buf = slice_from_raw_parts(&tmp_stat as *const _ as *const u8, size_of::<Stat>());
    for (i, byte) in user_buffer.into_iter().enumerate() {
        unsafe {
            *byte = (*stat_buf)[i];
        }
    }
    0
}
