mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub use address::{PhysPageNum, VirtAddr};
pub use memory_set::KERNEL_SPACE;
pub use memory_set::{MapPermission, MemorySet, remap_test};
pub use page_table::{translated_byte_buffer, translated_refmut, translated_str};

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}
