#[allow(unused)]

/// 用户应用程序的堆栈大小
pub const USER_STACK_SIZE: usize = 4096 * 2;
/// 内核堆栈大小
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
/// 内核堆大小
pub const KERNEL_HEAP_SIZE: usize = 0x20_0000;

/// page size : 4KB
pub const PAGE_SIZE: usize = 0x1000;
/// page size bits: 12
pub const PAGE_SIZE_BITS: usize = 0xc;
/// 跳板的虚拟地址
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
/// trap上下文的虚拟地址
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

pub const CLOCK_FREQ: usize = 12500000;
pub const MEMORY_END: usize = 0x8800_0000;

pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x1000_1000, 0x00_1000), // Virtio Block in virt machine
];

