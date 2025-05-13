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
use user_lib::{event_get, get_time};
use virtio_input_decoder::Key;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    let size = Size::new(VIRTGPU_XRES, VIRTGPU_YRES);
    let mut disp = Display::new(size);
    let mut wm = WindowManager::new(size);

    let desktop_buffer = create_desktop_background(size);

    wm.add_desktop_icon(Point::new(50, 50), "move");
    wm.add_desktop_icon(Point::new(50, 150), "shape");
    wm.add_desktop_icon(Point::new(50, 250), "snake");

    let mut last_update_times = alloc::collections::BTreeMap::new();

    loop {
        while let Some(raw_evt) = event_get() {
            if let Some(win_evt) = translate_event(raw_evt) {
                if matches!(win_evt, WindowEvent::KeyPress(Key::ESC)) {
                    return 0;
                }
                wm.handle_event(win_evt);
            }
        }

        let now = get_time();
        for window in wm.get_windows_mut() {
            if let Some(app) = &window.app {
                if app.needs_timer() {
                    let interval = app.timer_interval() as isize;
                    let last_time = last_update_times.get(&window.id).copied().unwrap_or(0);
                    if now - last_time >= interval {
                        let updated = window.update_app();
                        if updated {
                            // println!("App updated");
                        }
                        last_update_times.insert(window.id, now);
                    }
                }
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

    let gradient_style = PrimitiveStyleBuilder::new()
        .fill_color(Rgb888::new(50, 100, 150))
        .build();

    Rectangle::new(Point::zero(), size)
        .into_styled(gradient_style)
        .draw(&mut buffer)
        .unwrap();

    buffer
}
