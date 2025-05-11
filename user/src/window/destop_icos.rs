use super::buffer::EmbeddedGraphicsBuffer;
use alloc::string::{String, ToString};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::iso_8859_10::FONT_6X10;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;

pub struct DesktopIcon {
    pub id: usize,
    pub position: Point,
    pub size: Size,
    pub label: String,
    pub icon_buffer: EmbeddedGraphicsBuffer,
    pub is_selected: bool,
}

impl DesktopIcon {
    pub fn new(id: usize, position: Point, label: &str) -> Self {
        let icon_size = Size::new(48, 48);

        Self {
            id,
            position,
            size: Size::new(64, 64), // 图标加标签的总大小
            label: label.to_string(),
            icon_buffer: EmbeddedGraphicsBuffer::new(icon_size),
            is_selected: false,
        }
    }

    pub fn render<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb888>,
    {
        // 绘制图标
        self.icon_buffer.blit_to(target, self.position)?;

        // 绘制标签
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::CSS_WHITE);
        Text::new(
            &self.label,
            Point::new(
                self.position.x + (self.size.width as i32 - self.label.len() as i32 * 6) / 2,
                self.position.y + 48 + 10,
            ),
            text_style,
        )
        .draw(target)?;

        // 如果被选中，绘制选中效果
        if self.is_selected {
            Rectangle::new(
                Point::new(self.position.x - 2, self.position.y - 2),
                Size::new(self.size.width + 4, self.size.height + 4),
            )
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::CSS_BLUE, 2))
            .draw(target)?;
        }

        Ok(())
    }

    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.position.x
            && x <= self.position.x + self.size.width as i32
            && y >= self.position.y
            && y <= self.position.y + self.size.height as i32
    }
}
