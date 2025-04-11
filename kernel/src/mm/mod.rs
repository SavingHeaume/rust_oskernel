mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub use address::{PhysPageNum, VirtAddr, PhysAddr, StepByOne};
pub use frame_allocator::{FrameTracker, frame_alloc, frame_dealloc};
pub use memory_set::{KERNEL_SPACE, kernel_token};
pub use memory_set::{MapPermission, MemorySet, remap_test};
pub use page_table::{translated_byte_buffer, translated_refmut, translated_str, PageTable};

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}
