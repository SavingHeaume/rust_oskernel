use super::event::WindowEvent;

pub struct MouseState {
    last_x: i32,
    last_y: i32,
    buttons: u8,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            last_x: 0,
            last_y: 0,
            buttons: 0,
        }
    }

    /// 更新鼠标状态并返回有效事件
    pub fn update(&mut self, event: WindowEvent) -> Option<WindowEvent> {
        match event {
            WindowEvent::MouseMove { x, y } => {
                // 合并X/Y轴事件
                let new_x = if x != 0 { x } else { self.last_x };
                let new_y = if y != 0 { y } else { self.last_y };
                let delta = (new_x != self.last_x) || (new_y != self.last_y);

                self.last_x = new_x;
                self.last_y = new_y;

                delta.then(|| WindowEvent::MouseMove { x: new_x, y: new_y })
            }
            WindowEvent::MousePress { button, .. } => {
                self.buttons |= 1 << button as u8;
                Some(event)
            }
            WindowEvent::MouseRelease { button } => {
                self.buttons &= !(1 << button as u8);
                Some(event)
            }
            _ => Some(event),
        }
    }
}
