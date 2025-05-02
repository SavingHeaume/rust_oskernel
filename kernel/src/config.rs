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
pub const TRAP_CONTEXT_BASE: usize = TRAMPOLINE - PAGE_SIZE;

pub const CLOCK_FREQ: usize = 12500000;
pub const MEMORY_END: usize = 0x8800_0000;

pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x2000000, 0x10000),
    (0xc000000, 0x210000), // VIRT_PLIC in virt machine
    (0x10000000, 0x9000),  // VIRT_UART0 with GPU  in virt machine
];

pub const VIRT_PLIC: usize = 0xC00_0000;
pub const VIRT_UART: usize = 0x1000_0000;
#[allow(unused)]
pub const VIRTGPU_XRES: u32 = 1280;
#[allow(unused)]
pub const VIRTGPU_YRES: u32 = 800;
