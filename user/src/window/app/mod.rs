mod movement;
mod shape;
mod snake;

use crate::window::EmbeddedGraphicsBuffer;
use crate::window::WindowEvent;
use alloc::boxed::Box;
use core::convert::Infallible;
use embedded_graphics::prelude::*;

/// 应用程序接口，所有要在桌面窗口中运行的应用必须实现这个特性
pub trait AppInterface {
    /// 初始化应用程序，返回程序名称
    fn init(&mut self, size: Size) -> &str;

    /// 处理窗口事件
    fn handle_event(&mut self, event: WindowEvent);

    /// 渲染应用程序到缓冲区
    fn render(&mut self, buffer: &mut EmbeddedGraphicsBuffer) -> Result<(), Infallible>;

    /// 更新应用程序状态，返回是否需要重新渲染
    fn update(&mut self) -> bool;

    /// 获取应用是否需要定期更新
    fn needs_timer(&self) -> bool {
        false
    }

    /// 获取定时器间隔（毫秒）
    fn timer_interval(&self) -> u64 {
        100
    }
}

// 全局应用程序注册表
pub struct AppRegistry;

impl AppRegistry {
    pub fn create_app(name: &str) -> Option<Box<dyn AppInterface>> {
        match name {
            "shape" => Some(Box::new(shape::ShapeApp::new())),
            "move" => Some(Box::new(movement::MoveApp::new())),
            "snake" => Some(Box::new(snake::SnakeApp::new())),
            _ => None,
        }
    }
}
