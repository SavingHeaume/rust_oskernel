use super::event::WindowEvent;
use super::window::Window;
use alloc::vec::Vec;
use embedded_graphics::geometry::Size;
use embedded_graphics::prelude::Point;
use virtio_input_decoder::Key;

pub struct WindowManager {
    windows: Vec<Window>,
    next_id: usize,
    screen_size: Size,
    drag_state: Option<DragState>,
    cursor_pos: Point,
}

struct DragState {
    window_id: usize,
    offset: (i32, i32), // 相对于窗口左上角的偏移
}

impl WindowManager {
    pub fn new(screen_size: Size) -> Self {
        Self {
            windows: Vec::new(),
            next_id: 1,
            screen_size,
            drag_state: None,
            cursor_pos: Point::zero(),
        }
    }

    /// 创建新窗口
    pub fn create_window(&mut self, pos: Point, size: Size, title: &str) -> usize {
        let id = self.next_id;
        self.windows.push(Window::new(id, pos, size, title));
        self.next_id += 1;
        id
    }

    /// 处理所有窗口的渲染
    pub fn render(&mut self, main_fb: &mut [u8]) {
        for window in &mut self.windows {
            if window.is_dirty {
                window.render(main_fb, self.screen_size);
                window.is_dirty = false;
            }
        }
    }

    pub fn handle_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::MouseMove { x, y } => {
                // 转换为相对坐标
                if let Some(drag) = &self.drag_state {
                    let window = self
                        .windows
                        .iter_mut()
                        .find(|w| w.id == drag.window_id)
                        .unwrap();
                    let new_x = x - drag.offset.0;
                    let new_y = y - drag.offset.1;
                    window.set_position(new_x, new_y);
                }
            }
            WindowEvent::MousePress {
                x,
                y,
                button: Key::MouseLeft,
            } => {
                if let Some(win_id) = self.find_window_at(x, y) {
                    self.bring_to_front(win_id);

                    let win = self.windows.iter_mut().find(|w| w.id == win_id).unwrap();

                    let offset_x = x - win.bounds.top_left.x;
                    let offset_y = y - win.bounds.top_left.y;
                    self.drag_state = Some(DragState {
                        window_id: win.id,
                        offset: (offset_x, offset_y),
                    });
                }
            }
            WindowEvent::MouseRelease {
                button: Key::MouseLeft,
            } => {
                self.drag_state = None;
            }
            WindowEvent::KeyPress(Key::MouseRight) => {
                self.close_top_window();
            }
            _ => {}
        }
    }

    fn find_window_at(&self, x: i32, y: i32) -> Option<usize> {
        self.windows
            .iter()
            .rev()
            .find(|w| w.contains_point(x, y))
            .map(|w| w.id)
    }

    pub fn close_top_window(&mut self) {
        if let Some(max_z) = self.windows.iter().map(|w| w.z_index).max() {
            self.windows.retain(|w| w.z_index != max_z);
        }
    }

    /// 更新窗口层级
    fn bring_to_front(&mut self, window_id: usize) {
        let max_z = self.windows.iter().map(|w| w.z_index).max().unwrap_or(0);
        if let Some(window) = self.windows.iter_mut().find(|w| w.id == window_id) {
            window.z_index = max_z + 1;
        }
    }

    /// 查找当前拖拽的窗口（实现示例）
    fn get_dragged_window_mut(&mut self) -> Option<&mut Window> {
        self.drag_state
            .as_ref()
            .and_then(|drag| self.windows.iter_mut().find(|w| w.id == drag.window_id))
    }

    pub fn cleanup(&mut self) {
        self.windows.retain(|w| w.is_alive);
    }
}
