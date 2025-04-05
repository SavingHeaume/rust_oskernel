use crate::BLOCK_SZ;
use crate::block_cache::get_block_cache;
use crate::block_dev::BlockDevice;
use alloc::sync::Arc;

const EFS_MAGIC: u32 = 0x3b800001;
const INODE_DIRECT_COUNT: usize = 28;
const INODE_INDIRECT1_COUNT: usize = BLOCK_SZ / 4;
const DIRECT_BOUND: usize = INODE_DIRECT_COUNT;
const INDIRECT1_BOUND: usize = DIRECT_BOUND + INODE_INDIRECT1_COUNT;
type IndirecBlock = [u32; BLOCK_SZ / 4];

#[repr(C)]
pub struct SuperBlock {
    magic: u32,
    pub total_blocks: u32,
    pub inode_bitmap_blocks: u32,
    pub inode_area_blocks: u32,
    pub data_bitmap_blocks: u32,
    pub data_area_blocks: u32,
}

impl SuperBlock {
    pub fn initialize(
        &mut self,
        total_blocks: u32,
        inode_bitmap_blocks: u32,
        inode_area_blocks: u32,
        data_bitmap_blocks: u32,
        data_area_blocks: u32,
    ) {
        *self = Self {
            magic: EFS_MAGIC,
            total_blocks,
            inode_bitmap_blocks,
            inode_area_blocks,
            data_bitmap_blocks,
            data_area_blocks,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == EFS_MAGIC
    }
}

#[repr(C)]
pub struct DiskInode {
    pub size: u32,
    pub direct: [u32; INODE_DIRECT_COUNT],
    pub indirct1: u32,
    pub indirct2: u32,
    type_: DiskInodeType,
}

#[derive(PartialEq)]
pub enum DiskInodeType {
    File,
    Directory,
}

impl DiskInode {
    pub fn initialize(&mut self, type_: DiskInodeType) {
        self.size = 0;
        self.direct.iter_mut().for_each(|v| *v = 0);
        self.indirct1 = 0;
        self.indirct2 = 0;
        self.type_ = type_;
    }

    pub fn is_dir(&self) -> bool {
        self.type_ == DiskInodeType::Directory
    }

    pub fn is_file(&self) -> bool {
        self.type_ == DiskInodeType::File
    }

    pub fn get_block_id(&self, inner_id: u32, block_device: &Arc<dyn BlockDevice>) -> u32 {
        let inner_id = inner_id as usize;
        if inner_id < INODE_DIRECT_COUNT {
            self.direct[inner_id]
        } else if inner_id < INDIRECT1_BOUND {
            get_block_cache(self.indirct1 as usize, Arc::clone(block_device))
                .lock()
                .read(0, |indirect_block: &IndirecBlock| {
                    indirect_block[inner_id - INODE_DIRECT_COUNT]
                })
        } else {
            let last = inner_id - INDIRECT1_BOUND;
            let indirec1 = get_block_cache(self.indirct2 as usize, Arc::clone(block_device))
                .lock()
                .read(0, |indirect2: &IndirecBlock| {
                    indirect2[last / INODE_INDIRECT1_COUNT]
                });

            get_block_cache(indirec1 as usize, Arc::clone(block_device))
                .lock()
                .read(0, |indirec1: &IndirecBlock| {
                    indirec1[last % INODE_DIRECT_COUNT]
                })
        }
    }
}
