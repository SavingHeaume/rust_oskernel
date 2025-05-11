#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use user_lib::window::{EmbeddedGraphicsBuffer, WindowEvent, WindowManager, translate_event};
use user_lib::{Display, VIRTGPU_XRES, VIRTGPU_YRES};
use user_lib::{event_get, sleep};
use virtio_input_decoder::{DecodeType, Key, KeyType, Mouse};

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    let size = Size::new(VIRTGPU_XRES, VIRTGPU_YRES);
    let mut disp = Display::new(size);
    let mut wm = WindowManager::new(size);

    let win1 = wm.create_window(Point::new(100, 100), Size::new(400, 300), "Window 1");
    let win2 = wm.create_window(Point::new(300, 200), Size::new(300, 200), "Window 2");

    let mut desktop_buffer = create_desktop_background(size);

    wm.add_desktop_icon(Point::new(50, 50), "file");
    wm.add_desktop_icon(Point::new(50, 150), "set");
    wm.add_desktop_icon(Point::new(50, 250), "shell");

    loop {
        while let Some(raw_evt) = event_get() {
            if let Some(win_evt) = translate_event(raw_evt) {
                if matches!(win_evt, WindowEvent::KeyPress(Key::ESC)) {
                    return 0;
                }
                wm.handle_event(win_evt);
            }
        }

        wm.cleanup();

        disp.clear(Rgb888::BLACK).unwrap();
        desktop_buffer.draw(&mut disp).unwrap();
        wm.render(&mut disp).unwrap();
        disp.flush();

        //sleep(16);
    }
}

/// 创建简单的桌面背景
fn create_desktop_background(size: Size) -> impl Drawable<Color = Rgb888> {
    let mut buffer = EmbeddedGraphicsBuffer::new(size);

    // 绘制渐变背景 (简化版)
    let gradient_style = PrimitiveStyleBuilder::new()
        .fill_color(Rgb888::new(50, 100, 150))
        .build();

    Rectangle::new(Point::zero(), size)
        .into_styled(gradient_style)
        .draw(&mut buffer)
        .unwrap();

    // 可以在这里添加桌面图标等其他元素

    buffer
}
