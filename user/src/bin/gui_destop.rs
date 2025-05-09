#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::{Point, RgbColor, Size};
use user_lib::sleep;
use user_lib::{Display, VIRTGPU_XRES, VIRTGPU_YRES};
use user_lib::{WindowManager, event_get, translate_event};

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    let mut disp = Display::new(Size::new(VIRTGPU_XRES, VIRTGPU_YRES));
    let mut wm = WindowManager::new(Size::new(VIRTGPU_XRES, VIRTGPU_YRES));

    let win_id = wm.create_window(Point::new(100, 100), Size::new(400, 300), "demo window");

    loop {
        while let Some(raw_evt) = event_get() {
            if let Some(win_evt) = translate_event(raw_evt) {
                wm.handle_event(win_evt);
            }
        }

        wm.cleanup();

        disp.clear(Rgb888::BLACK).unwrap();
        wm.render(disp.framebuffer());
        disp.flush();

        sleep(16);
    }
}
