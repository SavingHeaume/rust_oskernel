use super::bus::virtio::VirtioHal;
use crate::sync::UPIntrFreeCell;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::any::Any;
use embedded_graphics::pixelcolor::Rgb888;
use tinybmp::Bmp;
use virtio_drivers::{VirtIOGpu, VirtIOHeader};

const VIRTIO7: usize = 0x10007000;

pub trait GpuDevice: Send + Sync + Any {
    fn get_framebuffer(&self) -> &mut [u8];
    fn flush(&self);
}

pub struct VirtIOGpuWrapper {
    gpu: UPIntrFreeCell<VirtIOGpu<'static, VirtioHal>>,
    fb: &'static [u8],
}

lazy_static::lazy_static! {
    pub static ref GPU_DEVICE: Arc<dyn GpuDevice> = Arc::new(VirtIOGpuWrapper::new());
}

impl GpuDevice for VirtIOGpuWrapper {
    fn flush(&self) {
        self.gpu.exclusive_access().flush().unwrap();
    }

    fn get_framebuffer(&self) -> &mut [u8] {
        unsafe {
            let ptr = self.fb.as_ptr() as *const _ as *mut u8;
            core::slice::from_raw_parts_mut(ptr, self.fb.len())
        }
    }
}

static BMP_DATA: &[u8] = include_bytes!("../../assert/mouse.bmp");
impl VirtIOGpuWrapper {
    pub fn new() -> Self {
        unsafe {
            // 执行virtio_drivers中的基本初始化

            let mut virtio =
                VirtIOGpu::<VirtioHal>::new(&mut *(VIRTIO7 as *mut VirtIOHeader)).unwrap();
            // 设置显存
            let fbuffer = virtio.setup_framebuffer().unwrap();

            let len = fbuffer.len();
            let ptr = fbuffer.as_mut_ptr();
            let fb = core::slice::from_raw_parts_mut(ptr, len);
            // 初始化光标
            let bmp = Bmp::<Rgb888>::from_slice(BMP_DATA).unwrap();
            let raw = bmp.as_raw();
            let mut b = Vec::new();
            for i in raw.image_data().chunks(3) {
                let mut v = i.to_vec();
                b.append(&mut v);
                if i == [255, 255, 255] {
                    b.push(0x0)
                } else {
                    b.push(0xff)
                }
            }
            virtio.setup_cursor(b.as_slice(), 50, 50, 50, 50).unwrap();

            Self {
                gpu: UPIntrFreeCell::new(virtio),
                fb,
            }
        }
    }
}
