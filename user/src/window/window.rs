use super::WindowEvent;
use super::app::AppInterface;
use super::buffer::EmbeddedGraphicsBuffer;
use super::style::WindowStyle;
use crate::io::{VIRTGPU_XRES, VIRTGPU_YRES};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use core::convert::Infallible;
use embedded_graphics::mono_font::{MonoTextStyle, ascii::FONT_6X10};
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::PrimitiveStyle;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::Text;

pub struct Window {
    pub id: usize,
    pub bounds: Rectangle, // 窗口位置和大小 (逻辑坐标)
    pub title: String,
    pub z_index: i32,
    pub is_dirty: bool, // 是否需要重绘
    pub is_alive: bool, // 窗口是否存活
    pub style: WindowStyle,
    pub buffer: EmbeddedGraphicsBuffer,

    pub content_buffer: Option<EmbeddedGraphicsBuffer>,
    pub content_offset: Point,

    pub app: Option<Box<dyn AppInterface>>,
    pub app_size: Size,
}

impl Window {
    pub fn new(id: usize, position: Point, size: Size, title: &str) -> Self {
        let style = WindowStyle::default();
        let buffer_size = Size::new(
            size.width + 2 * style.border_width,
            size.height + style.titlebar_height + style.border_width,
        );

        Self {
            id,
            bounds: Rectangle::new(position, buffer_size),
            style, // 这里将 style 存入结构体字段
            title: title.to_string(),
            buffer: EmbeddedGraphicsBuffer::new(buffer_size),
            z_index: 0,
            is_dirty: true,
            is_alive: true,

            content_buffer: None,
            content_offset: Point::zero(),

            app: None,
            app_size: Size::new(0, 0),
        }
    }

    /// 主渲染入口
    pub fn render<D>(&mut self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb888>,
    {
        if self.is_dirty {
            self.redraw_decorations().unwrap();
        }
        self.buffer.blit_to(target, self.bounds.top_left)?;

        let top_left = self.content_area().top_left;

        if let (Some(app), Some(buffer)) = (&mut self.app, &mut self.content_buffer) {
            app.render(buffer).ok();
            buffer.blit_to(target, top_left)?;
        }

        Ok(())
    }

    /// 重绘装饰元素
    fn redraw_decorations(&mut self) -> Result<(), <EmbeddedGraphicsBuffer as DrawTarget>::Error> {
        // 清空缓冲
        self.buffer.clear(Rgb888::BLACK)?;

        // 绘制边框
        Rectangle::new(Point::zero(), self.buffer.size)
            .into_styled(PrimitiveStyle::with_stroke(
                self.style.border_color,
                self.style.border_width,
            ))
            .draw(&mut self.buffer)?;

        // 绘制标题栏背景
        let title_rect = Rectangle::new(
            Point::new(
                self.style.border_width as i32,
                self.style.border_width as i32,
            ),
            Size::new(
                self.buffer.size.width - 2 * self.style.border_width,
                self.style.titlebar_height,
            ),
        );
        title_rect
            .into_styled(PrimitiveStyle::with_fill(self.style.titlebar_color))
            .draw(&mut self.buffer)?;

        // 绘制标题文字
        let text_style = MonoTextStyle::new(&FONT_6X10, self.style.title_text_color);
        Text::new(
            &self.title,
            Point::new(
                self.style.border_width as i32 + 4,
                (self.style.border_width + (self.style.titlebar_height - 10) / 2) as i32,
            ),
            text_style,
        )
        .draw(&mut self.buffer)?;

        // 关闭按钮
        let close_btn_size = (self.style.titlebar_height - 6) as u32;
        let close_btn_x = self.buffer.size.width - self.style.border_width - close_btn_size - 3;
        let close_btn_y = self.style.border_width + 3;

        let close_btn = Rectangle::new(
            Point::new(close_btn_x as i32, close_btn_y as i32),
            Size::new(close_btn_size, close_btn_size),
        );

        close_btn
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(255, 80, 80)))
            .draw(&mut self.buffer)?;

        Ok(())
    }
    pub fn set_position(&mut self, new_x: i32, new_y: i32) {
        // 限制在屏幕范围内
        let max_x = (VIRTGPU_XRES as i32) - self.bounds.size.width as i32;
        let max_y = (VIRTGPU_YRES as i32) - self.bounds.size.height as i32;

        self.bounds.top_left.x = new_x.clamp(0, max_x);
        self.bounds.top_left.y = new_y.clamp(0, max_y);
        self.is_dirty = true;
    }

    /// 检测坐标是否在窗口内
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        self.bounds.contains(Point::new(x, y))
    }

    pub fn close(&mut self) {
        self.is_alive = false;
    }

    pub fn content_area(&self) -> Rectangle {
        let content_x = self.bounds.top_left.x + self.style.border_width as i32;
        let content_y =
            self.bounds.top_left.y + (self.style.border_width + self.style.titlebar_height) as i32;
        let content_width = self.bounds.size.width - 2 * self.style.border_width;
        let content_height =
            self.bounds.size.height - self.style.titlebar_height - 2 * self.style.border_width;

        Rectangle::new(
            Point::new(content_x, content_y),
            Size::new(content_width, content_height),
        )
    }

    // 添加绘制内容的方法
    pub fn draw_content<F>(&mut self, draw_fn: F) -> Result<(), Infallible>
    where
        F: FnOnce(&mut EmbeddedGraphicsBuffer) -> Result<(), Infallible>,
    {
        // 确保内容缓冲区存在
        if self.content_buffer.is_none() {
            let content_size = Size::new(
                self.bounds.size.width - 2 * self.style.border_width,
                self.bounds.size.height - self.style.titlebar_height - self.style.border_width,
            );
            self.content_buffer = Some(EmbeddedGraphicsBuffer::new(content_size));
            self.content_offset = Point::new(
                self.style.border_width as i32,
                (self.style.titlebar_height + self.style.border_width) as i32,
            );
        }

        // 应用绘制函数
        if let Some(buffer) = &mut self.content_buffer {
            draw_fn(buffer)?;
        }

        self.is_dirty = true;
        Ok(())
    }

    // 检查是否点击了关闭按钮
    pub fn is_close_button_clicked(&self, x: i32, y: i32) -> bool {
        let rel_x = x - self.bounds.top_left.x;
        let rel_y = y - self.bounds.top_left.y;

        let close_btn_size = (self.style.titlebar_height - 6) as i32;
        let close_btn_x =
            self.buffer.size.width as i32 - self.style.border_width as i32 - close_btn_size - 3;
        let close_btn_y = self.style.border_width as i32 + 3;

        rel_x >= close_btn_x
            && rel_x <= close_btn_x + close_btn_size
            && rel_y >= close_btn_y
            && rel_y <= close_btn_y + close_btn_size
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        // 确保最小尺寸
        let width = new_width.max(100);
        let height = new_height.max(50);

        // 更新窗口尺寸
        self.bounds.size.width = width;
        self.bounds.size.height = height;

        // 创建新的缓冲区
        self.buffer = EmbeddedGraphicsBuffer::new(Size::new(width, height));

        // 如果有内容缓冲区，也需要更新
        if let Some(content_buffer) = &mut self.content_buffer {
            let content_width = width - 2 * self.style.border_width;
            let content_height = height - self.style.titlebar_height - self.style.border_width;
            *content_buffer = EmbeddedGraphicsBuffer::new(Size::new(content_width, content_height));
        }

        // 标记为需要重绘
        self.is_dirty = true;
    }

    pub fn attach_app(&mut self, mut app: Box<dyn AppInterface>) {
        let content_area = self.content_area().size;
        self.app_size = content_area;

        app.init(content_area);
        self.app = Some(app);

        self.content_buffer = Some(EmbeddedGraphicsBuffer::new(content_area));
        self.is_dirty = true;
    }

    pub fn handle_app_event(&mut self, event: WindowEvent) {
        if let Some(app) = &mut self.app {
            app.handle_event(event);
            self.is_dirty = true;
        }
    }

    pub fn update_app(&mut self) -> bool {
        if let Some(app) = &mut self.app {
            if app.update() {
                self.is_dirty = true;
                return true;
            }
        }
        false
    }
}
