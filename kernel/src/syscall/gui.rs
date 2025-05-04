use crate::{
    drivers::gpu::GPU_DEVICE,
    mm::{MapArea, MapPermission, MapType, PhysAddr, VirtAddr},
    task::current_process,
};

// 显存的用户态起始虚拟地址
const FB_VADDR: usize = 0x10000000;

pub fn sys_framebuffer() -> isize {
    //获取显存的其实物理页帧和偏移
    let fb = GPU_DEVICE.get_framebuffer();
    let len = fb.len();

    let fb_start_pa = PhysAddr::from(fb.as_ptr() as usize);
    assert!(fb_start_pa.aligned());
    let fb_start_ppn = fb_start_pa.floor();
    let fb_start_vpn = VirtAddr::from(FB_VADDR).floor();
    let pn_offset = fb_start_ppn.0 as isize - fb_start_vpn.0 as isize;

    // 获取当前进程的地址空间
    let current_process = current_process();
    let mut inner = current_process.inner_exclusive_access();

    // 把显存的物理页帧映射到起始地址为FB_VADDR的用户态虚拟地址空间
    inner.memory_set.push(
        MapArea::new(
            (FB_VADDR as usize).into(),
            (FB_VADDR + len as usize).into(),
            MapType::Linear(pn_offset),
            MapPermission::R | MapPermission::W | MapPermission::U,
        ),
        None,
    );

    FB_VADDR as isize
}

pub fn sys_framebuffer_fluse() -> isize {
    GPU_DEVICE.flush();
    0
}
