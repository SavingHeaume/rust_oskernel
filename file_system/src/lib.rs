#![no_std]
extern crate alloc;
mod bitmap;
mod block_cache;
mod block_dev;
mod fs;
mod layout;
mod vfs;

pub const BLOCK_SZ: usize = 512;
use bitmap::Bitmap;
use block_cache::{block_cache_sync_all, get_block_cache};
pub use block_dev::BlockDevice;
pub use fs::FileSystem;
use layout::*;
pub use vfs::Inode;
pub use layout::DiskInodeType;
