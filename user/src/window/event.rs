use crate::io::InputEvent;
use virtio_input_decoder::{DecodeType, Key, KeyType, Mouse};

#[derive(Debug, Clone, Copy)]
pub enum WindowEvent {
    // 鼠标事件 (坐标已转换为屏幕绝对坐标)
    MouseMove { x: i32, y: i32 },
    MousePress { x: i32, y: i32, button: Key },
    MouseRelease { button: Key },

    // 键盘事件
    KeyPress(Key),
    KeyRelease(Key),

    // 窗口系统事件
    CloseRequest,
}

/// 将原始 InputEvent 转换为窗口系统事件
pub fn translate_event(input: InputEvent) -> Option<WindowEvent> {
    match input.decode() {
        Some(DecodeType::Key(key, key_type)) => match key_type {
            KeyType::Press => Some(WindowEvent::KeyPress(key)),
            KeyType::Release => Some(WindowEvent::KeyRelease(key)),
        },
        Some(DecodeType::Mouse(mouse)) => {
            match mouse {
                Mouse::X(x) => Some(WindowEvent::MouseMove { x: x as i32, y: 0 }), // 需结合Y坐标
                Mouse::Y(y) => Some(WindowEvent::MouseMove { x: 0, y: y as i32 }), // 需维护状态
                Mouse::ScrollUp => None, // 滚动事件单独处理
                Mouse::ScrollDown => None,
                _ => None,
            }
        }
        None => None,
    }
}
