use crate::{BLOCK_SZ, block_dev::BlockDevice};
use alloc::sync::Arc;

pub struct BlockCache {
    cache: [u8; BLOCK_SZ],
    block_id: usize,
    block_device: Arc<dyn BlockDevice>,
    modified: bool,
}
