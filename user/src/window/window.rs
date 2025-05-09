use crate::io::{VIRTGPU_XRES, VIRTGPU_YRES};
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use embedded_graphics::{
    geometry::{Point, Size},
    primitives::Rectangle,
};

pub struct Window {
    pub id: usize,
    pub bounds: Rectangle, // 窗口位置和大小 (逻辑坐标)
    pub title: String,
    pub content_buf: Vec<u8>, // 窗口内容缓冲
    pub z_index: i32,
    pub is_dirty: bool, // 是否需要重绘
    pub is_alive: bool, // 窗口是否存活
}

impl Window {
    pub fn new(id: usize, position: Point, size: Size, title: &str) -> Self {
        Self {
            id,
            bounds: Rectangle::new(position, size),
            title: title.to_string(),
            content_buf: vec![0; (size.width * size.height * 4) as usize],
            z_index: 0,
            is_dirty: true,
            is_alive: true,
        }
    }

    // 将窗口内容绘制到主显示缓冲
    pub fn render(&self, main_fb: &mut [u8], screen_size: Size) {
        let win_rect = self.bounds;
        for y in 0..win_rect.size.height {
            for x in 0..win_rect.size.width {
                // 计算屏幕位置
                let screen_x = win_rect.top_left.x + x as i32;
                let screen_y = win_rect.top_left.y + y as i32;

                // 边界检查
                if screen_x < 0
                    || screen_y < 0
                    || screen_x >= screen_size.width as i32
                    || screen_y >= screen_size.height as i32
                {
                    continue;
                }

                // 计算缓冲索引
                let content_idx = ((y * win_rect.size.width + x) * 4) as usize;
                let screen_idx = ((screen_y * screen_size.width as i32 + screen_x) * 4) as usize;

                // 复制像素数据 (ARGB格式)
                main_fb[screen_idx..screen_idx + 4]
                    .copy_from_slice(&self.content_buf[content_idx..content_idx + 4]);
            }
        }
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
}
