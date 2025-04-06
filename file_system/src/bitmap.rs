use crate::{BLOCK_SZ, bitmap, block_cache::get_block_cache, block_dev::BlockDevice};
use alloc::sync::Arc;

const BLOCK_BITS: usize = BLOCK_SZ * 8;
type BitmapBlock = [u64; 64];

pub struct Bitmap {
    start_block_id: usize, // 起始块编号
    blocks: usize,         // 长度为多少个区块
}

impl Bitmap {
    pub fn new(start_block_id: usize, blocks: usize) -> Self {
        Self {
            start_block_id,
            blocks,
        }
    }

    // 遍历区域中的每个块，再在每个块中以bit组（每组 64 bits）为单位进行遍历，
    // 找到一个尚未被全部分配出去的组，最后在里面分配一个bit。
    // 返回分配的bit所在的位置，等同于索引节点/数据块的编号。
    // 如果所有bit均已经被分配出去了，则返回 None 。
    pub fn alloc(&self, block_device: &Arc<dyn BlockDevice>) -> Option<usize> {
        for block_id in 0..self.blocks {
            let pos = get_block_cache(
                block_id + self.start_block_id as usize,
                Arc::clone(block_device),
            )
            .lock()
            // 这里 modify 的含义就是：
            // 从缓冲区偏移量为 0 的位置开始将一段连续的数据（数据的长度随具体类型而定）
            // 解析为一个 BitmapBlock 并要对该数据结构进行修改
            .modify(0, |bitmap_block: &mut BitmapBlock| {
                // 遍历每 64 bits构成的组（一个 u64 ），如果它并没有达到 u64::MAX，
                // 则通过 u64::trailing_ones 找到最低的一个 0 并置为 1
                // 如果能够找到的话，bit组的编号将保存在变量 bits64_pos 中，
                // 而分配的bit在组内的位置将保存在变量 inner_pos 中
                if let Some((bits64_pos, inner_pos)) = bitmap_block
                    .iter()
                    .enumerate()
                    .find(|(_, bits64)| **bits64 != u64::MAX)
                    .map(|(bits64_pos, bits64)| (bits64_pos, bits64.trailing_ones() as usize))
                {
                    bitmap_block[bits64_pos] |= 1u64 << inner_pos;
                    Some(block_id * BLOCK_BITS + bits64_pos + inner_pos as usize)
                } else {
                    None
                }
            });
        }
        None
    }

    pub fn dealloc(&self, block_device: &Arc<dyn BlockDevice>, bit: usize) {
        let (block_pos, bit64_pos, inner_pos) = decomposition(bit);
        get_block_cache(block_pos + self.start_block_id, Arc::clone(block_device))
            .lock()
            .modify(0, |bitmap_block: &mut BitmapBlock| {
                assert!(bitmap_block[bit64_pos] & (1u64 << inner_pos) > 0);
                bitmap_block[bit64_pos] -= 1u64 << inner_pos;
            })
    }

    pub fn maximum(&self) -> usize {
        self.blocks * BLOCK_BITS
    }
}

fn decomposition(mut bit: usize) -> (usize, usize, usize) {
    let block_pos = bit / BLOCK_BITS;
    bit = bit % BLOCK_BITS;
    (block_pos, bit / 64, bit % 64)
}
