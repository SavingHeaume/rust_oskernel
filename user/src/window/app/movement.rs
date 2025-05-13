use super::AppInterface;
use crate::window::{EmbeddedGraphicsBuffer, WindowEvent};
use core::convert::Infallible;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use virtio_input_decoder::Key;

pub struct MoveApp {
    rect_pos: Point,
    rect_size: u32,
    size: Size,
}

impl MoveApp {
    pub fn new() -> Self {
        Self {
            rect_pos: Point::new(100, 100),
            rect_size: 40,
            size: Size::new(0, 0),
        }
    }
    
    fn move_rect(&mut self, dx: i32, dy: i32) -> bool {
        let new_x = self.rect_pos.x + dx;
        let new_y = self.rect_pos.y + dy;
        let r = (self.rect_size / 2) as i32;
        
        // 确保矩形不会超出窗口边界
        if new_x > r
            && new_x + r < (self.size.width as i32)
            && new_y > r
            && new_y + r < (self.size.height as i32)
        {
            self.rect_pos.x = new_x;
            self.rect_pos.y = new_y;
            return true;
        }
        false
    }
}

impl AppInterface for MoveApp {
    fn init(&mut self, size: Size) -> &str {
        self.size = size;
        // 初始化时将矩形放在窗口中央
        self.rect_pos.x = (size.width / 2) as i32;
        self.rect_pos.y = (size.height / 2) as i32;
        "Move Demo"
    }

    fn handle_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::KeyPress(key) => {
                match key {
                    Key::W => { self.move_rect(0, -10); },
                    Key::A => { self.move_rect(-10, 0); },
                    Key::S => { self.move_rect(0, 10); },
                    Key::D => { self.move_rect(10, 0); },
                    _ => {}
                }
            },
            _ => {}
        }
    }

    fn render(&mut self, buffer: &mut EmbeddedGraphicsBuffer) -> Result<(), Infallible> {
        // 清空缓冲区
        buffer.clear(Rgb888::BLACK)?;
        
        // 绘制矩形
        Rectangle::with_center(self.rect_pos, Size::new(self.rect_size, self.rect_size))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 1))
            .draw(buffer)?;
            
        Ok(())
    }

    fn update(&mut self) -> bool {
        // 移动应用只在事件触发时更新
        false
    }
}
