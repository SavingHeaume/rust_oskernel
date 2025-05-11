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
                let new_x = if x != 0 { x + self.last_x } else { self.last_x };
                let new_y = if y != 0 { y + self.last_y } else { self.last_y };

                // 只有当坐标真正变化时才生成事件
                let has_changed = new_x != self.last_x || new_y != self.last_y;

                // 始终更新最后的坐标，确保状态一致性
                self.last_x = new_x;
                self.last_y = new_y;

                // 打印调试信息
                println!(
                    "MouseState: Position updated to ({}, {}), changed: {}",
                    new_x, new_y, has_changed
                );

                if has_changed {
                    // 返回完整的移动事件，包含两个坐标轴的信息
                    Some(WindowEvent::MouseMove { x: new_x, y: new_y })
                } else {
                    None
                }
            }
            WindowEvent::MousePress { button, .. } => {
                println!(
                    "MouseState: Button press {:?} at ({}, {})",
                    button, self.last_x, self.last_y
                );

                // 更新按钮状态
                self.buttons |= 1 << (button as u8);

                // 使用最后存储的坐标创建完整的按下事件
                Some(WindowEvent::MousePress {
                    x: self.last_x,
                    y: self.last_y,
                    button,
                })
            }
            WindowEvent::MouseRelease { button } => {
                println!("MouseState: Button release {:?}", button);

                // 更新按钮状态
                self.buttons &= !(1 << (button as u8));

                // 返回事件，不进行修改
                Some(event)
            }
            // 所有其他事件不变
            _ => Some(event),
        }
    }

    // 添加获取当前鼠标状态的辅助方法
    pub fn get_position(&self) -> (i32, i32) {
        (self.last_x, self.last_y)
    }

    pub fn is_button_pressed(&self, button: u8) -> bool {
        (self.buttons & (1 << button)) != 0
    }
}
