use super::{
    BlockDevice, DIRENT_SZ, DirEntry, DiskInode, DiskInodeType, FileSystem, block_cache_sync_all,
    get_block_cache,
};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::{Mutex, MutexGuard};

pub struct Inode {
    block_id: usize,
    block_offset: usize,
    fs: Arc<Mutex<FileSystem>>,
    block_device: Arc<dyn BlockDevice>,
}

impl Inode {
    pub fn new(
        block_id: u32,
        block_offset: usize,
        fs: Arc<Mutex<FileSystem>>,
        block_device: Arc<dyn BlockDevice>,
    ) -> Self {
        Self {
            block_id: block_id as usize,
            block_offset,
            fs,
            block_device,
        }
    }

    fn read_disk_inode<V>(&self, f: impl FnOnce(&DiskInode) -> V) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .read(self.block_offset, f)
    }

    fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut DiskInode) -> V) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .modify(self.block_offset, f)
    }

    fn find_inode_id(&self, name: &str, disk_inode: &DiskInode) -> Option<u32> {
        // assert it is a directory
        assert!(disk_inode.is_dir());
        let file_count = (disk_inode.size as usize) / DIRENT_SZ;
        let mut dirent = DirEntry::empty();
        for i in 0..file_count {
            assert_eq!(
                disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device,),
                DIRENT_SZ,
            );
            if dirent.name() == name {
                return Some(dirent.inode_number() as u32);
            }
        }
        None
    }

    pub fn find(&self, path: &str) -> Option<Arc<Inode>> {
        let fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            self.find_inode_id(path, disk_inode).map(|inode_id| {
                let (block_id, block_offset) = fs.get_disk_inode_pos(inode_id);
                Arc::new(Self::new(
                    block_id,
                    block_offset,
                    self.fs.clone(),
                    self.block_device.clone(),
                ))
            })
        })
    }

    fn increase_size(
        &self,
        new_size: u32,
        disk_inode: &mut DiskInode,
        fs: &mut MutexGuard<FileSystem>,
    ) {
        if new_size < disk_inode.size {
            return;
        }
        let blocks_needed = disk_inode.blocks_num_needed(new_size);
        let mut v: Vec<u32> = Vec::new();
        for _ in 0..blocks_needed {
            v.push(fs.alloc_data());
        }
        disk_inode.increase_size(new_size, v, &self.block_device);
    }

    pub fn create(&self, name: &str) -> Option<Arc<Inode>> {
        self.create_inode(name, DiskInodeType::File)
    }

    pub fn create_dir(&self, name: &str) -> Option<Arc<Inode>> {
        self.create_inode(name, DiskInodeType::Directory)
    }

    pub fn create_inode(&self, name: &str, inode_type: DiskInodeType) -> Option<Arc<Inode>> {
        let mut fs = self.fs.lock();

        // 检查文件是否存在
        let op = |root_inode: &DiskInode| {
            assert!(root_inode.is_dir());

            self.find_inode_id(name, root_inode)
        };
        if self.read_disk_inode(op).is_some() {
            return None;
        }

        // 创建一个新inode并初始化
        let new_inode_id = fs.alloc_inode();
        let (new_inode_block_id, new_inode_block_offset) = fs.get_disk_inode_pos(new_inode_id);
        get_block_cache(new_inode_block_id as usize, Arc::clone(&self.block_device))
            .lock()
            .modify(new_inode_block_offset, |new_inode: &mut DiskInode| {
                new_inode.initialize(inode_type);
            });

        // 将新文件的目录项插入根目录
        self.modify_disk_inode(|root_inode| {
            let file_count = (root_inode.size as usize) / DIRENT_SZ;
            let new_size = (file_count + 1) * DIRENT_SZ;

            self.increase_size(new_size as u32, root_inode, &mut fs);

            let dirent = DirEntry::new(name, new_inode_id);
            root_inode.write_at(
                file_count * DIRENT_SZ,
                dirent.as_bytes(),
                &self.block_device,
            );
        });

        let (block_id, block_offset) = fs.get_disk_inode_pos(new_inode_id);
        block_cache_sync_all();

        Some(Arc::new(Self::new(
            block_id,
            block_offset,
            self.fs.clone(),
            self.block_device.clone(),
        )))
    }

    pub fn ls(&self) -> Vec<String> {
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            let mut v: Vec<String> = Vec::new();
            if disk_inode.is_file() {
                return v;
            }

            let file_count = (disk_inode.size as usize) / DIRENT_SZ;
            for i in 0..file_count {
                let mut dirent = DirEntry::empty();
                assert_eq!(
                    disk_inode.read_at(i * DIRENT_SZ, dirent.as_bytes_mut(), &self.block_device,),
                    DIRENT_SZ,
                );
                v.push(String::from(dirent.name()));
            }
            v
        })
    }

    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| disk_inode.read_at(offset, buf, &self.block_device))
    }

    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let mut fs = self.fs.lock();
        let size = self.modify_disk_inode(|disk_inode| {
            assert!(disk_inode.is_file());

            self.increase_size((offset + buf.len()) as u32, disk_inode, &mut fs);
            disk_inode.write_at(offset, buf, &self.block_device)
        });
        block_cache_sync_all();
        size
    }

    pub fn clear(&self) {
        let mut fs = self.fs.lock();
        self.modify_disk_inode(|disk_inode| {
            assert!(disk_inode.is_file());
            let size = disk_inode.size;
            let data_blocks_dealloc = disk_inode.clear_size(&self.block_device);
            assert!(data_blocks_dealloc.len() == DiskInode::total_blocks(size) as usize);
            for data_block in data_blocks_dealloc.into_iter() {
                fs.dealloc_data(data_block);
            }
        });
        block_cache_sync_all();
    }

    pub fn get_file_size(&self) -> u32 {
        self.read_disk_inode(|disk_inode| disk_inode.size)
    }

    pub fn get_inode_id(&self) -> u32 {
        self.fs
            .lock()
            .get_disk_inode_id(self.block_id as u32, self.block_offset)
    }

    pub fn is_dir(&self) -> bool {
        self.read_disk_inode(|disk_inode| disk_inode.is_dir())
    }

    pub fn is_file(&self) -> bool {
        self.read_disk_inode(|disk_inode| disk_inode.is_file())
    }

    pub fn delete(&self, name: &str) -> bool {
        let mut fs = self.fs.lock();

        // 首先检查当前 inode 是否为目录
        let is_current_dir = self.read_disk_inode(|disk_inode| disk_inode.is_dir());
        if !is_current_dir {
            return false; // 只能在目录中删除文件
        }

        // 查找要删除的文件/目录的 inode_id
        let target_inode_id =
            self.read_disk_inode(|disk_inode| self.find_inode_id(name, disk_inode));

        let target_inode_id = match target_inode_id {
            Some(id) => id,
            None => return false, // 文件不存在
        };

        // 获取目标 inode 的位置并检查其类型
        let (target_block_id, target_block_offset) = fs.get_disk_inode_pos(target_inode_id);

        // 检查目标是否为目录，如果是目录则检查是否为空
        let target_is_empty_dir =
            get_block_cache(target_block_id as usize, Arc::clone(&self.block_device))
                .lock()
                .read(target_block_offset, |target_disk_inode: &DiskInode| {
                    if target_disk_inode.is_dir() {
                        // 目录必须为空才能删除
                        target_disk_inode.size == 0
                    } else {
                        true // 文件可以直接删除
                    }
                });

        if !target_is_empty_dir {
            return false; // 目录不为空，不能删除
        }

        // 清理目标 inode 的数据块
        get_block_cache(target_block_id as usize, Arc::clone(&self.block_device))
            .lock()
            .modify(target_block_offset, |target_disk_inode: &mut DiskInode| {
                if target_disk_inode.is_file() {
                    // 释放文件的所有数据块
                    let size = target_disk_inode.size;
                    let data_blocks_dealloc = target_disk_inode.clear_size(&self.block_device);
                    assert!(data_blocks_dealloc.len() == DiskInode::total_blocks(size) as usize);
                    for data_block in data_blocks_dealloc.into_iter() {
                        fs.dealloc_data(data_block);
                    }
                }
                // 目录的话，由于已经检查为空，所以不需要释放数据块
            });

        // 释放目标 inode
        fs.dealloc_inode(target_inode_id);

        // 从当前目录中移除目录项
        self.modify_disk_inode(|root_inode| {
            let file_count = (root_inode.size as usize) / DIRENT_SZ;
            let mut found_index = None;

            // 找到要删除的目录项的索引
            for i in 0..file_count {
                let mut dirent = DirEntry::empty();
                assert_eq!(
                    root_inode.read_at(i * DIRENT_SZ, dirent.as_bytes_mut(), &self.block_device),
                    DIRENT_SZ,
                );
                if dirent.name() == name {
                    found_index = Some(i);
                    break;
                }
            }

            if let Some(index) = found_index {
                // 将最后一个目录项移动到被删除项的位置
                if index < file_count - 1 {
                    let mut last_dirent = DirEntry::empty();
                    assert_eq!(
                        root_inode.read_at(
                            (file_count - 1) * DIRENT_SZ,
                            last_dirent.as_bytes_mut(),
                            &self.block_device
                        ),
                        DIRENT_SZ,
                    );
                    root_inode.write_at(
                        index * DIRENT_SZ,
                        last_dirent.as_bytes(),
                        &self.block_device,
                    );
                }

                // 减少目录大小
                let new_size = (file_count - 1) * DIRENT_SZ;
                root_inode.size = new_size as u32;

                // 如果目录变小了很多，可以考虑释放一些数据块
                // 这里简化处理，不释放已分配的块
            }
        });

        // 同步所有缓存
        block_cache_sync_all();
        true
    }
}
