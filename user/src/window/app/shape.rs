use super::AppInterface;
use crate::window::{EmbeddedGraphicsBuffer, WindowEvent};
use core::convert::Infallible;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, Rectangle, Triangle};

pub struct ShapeApp {
    size: Size,
    offset_x: i32,
    offset_y: i32,
    initialized: bool,
}

impl ShapeApp {
    pub fn new() -> Self {
        Self {
            size: Size::new(0, 0),
            offset_x: 80,
            offset_y: 80,
            initialized: false,
        }
    }
}

impl AppInterface for ShapeApp {
    fn init(&mut self, size: Size) -> &str {
        self.size = size;
        self.initialized = true;
        "Shape Demo"
    }

    fn handle_event(&mut self, _event: WindowEvent) {
        // 形状演示应用不处理事件
    }

    fn render(&mut self, buffer: &mut EmbeddedGraphicsBuffer) -> Result<(), Infallible> {
        // 清空缓冲区
        buffer.clear(Rgb888::BLACK)?;

        let center = Point::new(self.offset_x, self.offset_y);

        // 绘制红色边框矩形
        Rectangle::with_center(center, Size::new(150, 150))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 10))
            .draw(buffer)?;

        // 蓝色圆形
        Circle::new(center + Point::new(-70, -70), 60)
            .into_styled(PrimitiveStyle::with_fill(Rgb888::BLUE))
            .draw(buffer)?;

        // 绿色三角形
        Triangle::new(
            center + Point::new(0, 80),
            center + Point::new(80, 130),
            center + Point::new(-80, 130),
        )
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 10))
        .draw(buffer)?;

        Ok(())
    }

    fn update(&mut self) -> bool {
        // 为了演示效果，可以让形状轻微移动
        if self.initialized {
            self.offset_x = (self.offset_x + 1) % (self.size.width as i32 - 100);
            true
        } else {
            false
        }
    }
    
    fn needs_timer(&self) -> bool {
        true
    }
    
    fn timer_interval(&self) -> u64 {
        300 // 缓慢移动
    }
}
