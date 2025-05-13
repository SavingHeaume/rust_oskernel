use super::app::AppRegistry;
use super::cursor::MouseState;
use super::destop_icos::DesktopIcon;
use super::event::WindowEvent;
use super::window::Window;
use alloc::string::String;
use alloc::vec::Vec;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::Point;
use virtio_input_decoder::Key;

pub struct WindowManager {
    windows: Vec<Window>,
    next_id: usize,
    screen_size: Size,
    drag_state: Option<DragState>,
    resize_state: Option<ResizeState>,
    pub desktop_icons: Vec<DesktopIcon>,
    mouse_state: MouseState,
}

struct DragState {
    window_id: usize,
    offset: (i32, i32), // 相对于窗口左上角的偏移
}

struct ResizeState {
    window_id: usize,
    edge: ResizeEdge,
    start_point: Point,
    start_size: Size,
}

enum ResizeEdge {
    Bottom,
    Right,
    BottomRight,
}

impl WindowManager {
    pub fn new(screen_size: Size) -> Self {
        Self {
            windows: Vec::new(),
            next_id: 1,
            screen_size,
            drag_state: None,
            resize_state: None,
            desktop_icons: Vec::new(),
            mouse_state: MouseState::new(),
        }
    }

    /// 创建新窗口
    pub fn create_window(&mut self, pos: Point, size: Size, title: &str) -> usize {
        let id = self.next_id;
        self.windows.push(Window::new(id, pos, size, title));
        self.next_id += 1;
        id
    }

    pub fn create_app_window(&mut self, app_name: &str) -> Option<usize> {
        println!("crate_app");
        if let Some(app) = AppRegistry::create_app(app_name) {
            let size = Size::new(400, 300);
            let id = self.create_window(Point::new(100, 100), size, app_name);

            if let Some(window) = self.windows.iter_mut().find(|w| w.id == id) {
                window.attach_app(app);
                self.bring_to_front(id);
                return Some(id);
            }
        }
        None
    }

    /// 处理所有窗口的渲染
    pub fn render<D>(&mut self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb888>,
    {
        for icon in &self.desktop_icons {
            icon.render(target)?;
        }

        // 按z-index排序，低层级先渲染
        self.windows.sort_by_key(|w| w.z_index);

        for window in &mut self.windows {
            window.render(target)?;
        }
        Ok(())
    }

    pub fn handle_event(&mut self, event: WindowEvent) {
        let processed_event = self.mouse_state.update(event);

        if processed_event.is_none() {
            return;
        }

        if let Some(event) = processed_event {
            if let Some(window) = self.get_focused_window_mut() {
                window.handle_app_event(event.clone());
            }

            match event {
                WindowEvent::MouseMove { x, y } => {
                    // 处理调整大小
                    if let Some(resize) = &self.resize_state {
                        if let Some(window) =
                            self.windows.iter_mut().find(|w| w.id == resize.window_id)
                        {
                            let dx = x - resize.start_point.x;
                            let dy = y - resize.start_point.y;

                            let mut new_width = resize.start_size.width;
                            let mut new_height = resize.start_size.height;

                            match resize.edge {
                                ResizeEdge::Bottom => {
                                    new_height =
                                        (resize.start_size.height as i32 + dy).max(50) as u32;
                                }
                                ResizeEdge::Right => {
                                    new_width =
                                        (resize.start_size.width as i32 + dx).max(50) as u32;
                                }
                                ResizeEdge::BottomRight => {
                                    new_width =
                                        (resize.start_size.width as i32 + dx).max(50) as u32;
                                    new_height =
                                        (resize.start_size.height as i32 + dy).max(50) as u32;
                                }
                            }

                            // 更新窗口大小
                            window.resize(new_width, new_height);
                            return;
                        }
                    }

                    // 检查拖拽状态
                    if let Some(drag) = &self.drag_state {
                        println!("Dragging window {}", drag.window_id);

                        if let Some(window) =
                            self.windows.iter_mut().find(|w| w.id == drag.window_id)
                        {
                            let new_x = x - drag.offset.0;
                            let new_y = y - drag.offset.1;

                            println!("Setting window position to ({}, {})", new_x, new_y);
                            window.set_position(new_x, new_y);
                        } else {
                            println!("Window not found!");
                        }
                    }

                    for icon in &mut self.desktop_icons {
                        icon.is_selected = false;
                    }
                }
                WindowEvent::MousePress {
                    x,
                    y,
                    button: Key::MouseLeft,
                } => {
                    if let Some(icon_id) = self
                        .desktop_icons
                        .iter()
                        .find(|i| i.contains_point(x, y))
                        .map(|i| i.id)
                    {
                        let mut name = String::new();
                        for icon in &mut self.desktop_icons {
                            if icon.id == icon_id {
                                icon.is_selected = true;
                                name = icon.label.clone();
                            }
                        }
                        self.create_app_window(name.as_str());
                    }

                    // 检测窗口关闭按钮
                    if let Some(win_id) = self.find_window_at(x, y) {
                        if let Some(window) = self.windows.iter_mut().find(|w| w.id == win_id) {
                            if window.is_close_button_clicked(x, y) {
                                window.close();
                                self.cleanup();
                                return; // 关闭窗口，不再处理后续逻辑
                            }
                        }
                    }

                    if let Some((win_id, edge)) = self.detect_resize_edge(x, y) {
                        if let Some(window) = self.windows.iter().find(|w| w.id == win_id) {
                            self.resize_state = Some(ResizeState {
                                window_id: win_id,
                                edge,
                                start_point: Point::new(x, y),
                                start_size: window.bounds.size,
                            });
                            return;
                        }
                    }

                    if let Some(win_id) = self.find_window_at(x, y) {
                        println!("Found window {} at position", win_id);
                        self.bring_to_front(win_id);

                        if let Some(win) = self.windows.iter().find(|w| w.id == win_id) {
                            let offset_x = x - win.bounds.top_left.x;
                            let offset_y = y - win.bounds.top_left.y;

                            println!(
                                "Setting drag state with offset ({}, {})",
                                offset_x, offset_y
                            );
                            self.drag_state = Some(DragState {
                                window_id: win.id,
                                offset: (offset_x, offset_y),
                            });
                        }
                    }
                }
                WindowEvent::MouseRelease {
                    button: Key::MouseLeft,
                } => {
                    self.resize_state = None;
                    self.drag_state = None;
                }
                _ => {}
            }
        }
    }

    fn find_window_at(&self, x: i32, y: i32) -> Option<usize> {
        // 创建窗口引用的临时列表并按 z_index 降序排序
        let mut windows: Vec<&Window> = self.windows.iter().collect();
        windows.sort_by_key(|w| -(w.z_index));

        // 查找包含坐标的顶层窗口
        windows
            .iter()
            .find(|&&w| w.contains_point(x, y))
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

    // 检测是否在窗口边缘
    fn detect_resize_edge(&self, x: i32, y: i32) -> Option<(usize, ResizeEdge)> {
        // 从顶层窗口开始检查
        for window in self.windows.iter().rev() {
            let edge_size = 8; // 边缘检测区域大小

            let is_bottom = y >= window.bounds.bottom_right().unwrap().y - edge_size
                && y <= window.bounds.bottom_right().unwrap().y
                && x >= window.bounds.top_left.x
                && x <= window.bounds.bottom_right().unwrap().x;

            let is_right = x >= window.bounds.bottom_right().unwrap().x - edge_size
                && x <= window.bounds.bottom_right().unwrap().x
                && y >= window.bounds.top_left.y
                && y <= window.bounds.bottom_right().unwrap().y;

            if is_bottom && is_right {
                return Some((window.id, ResizeEdge::BottomRight));
            } else if is_bottom {
                return Some((window.id, ResizeEdge::Bottom));
            } else if is_right {
                return Some((window.id, ResizeEdge::Right));
            }
        }

        None
    }

    pub fn add_desktop_icon(&mut self, position: Point, label: &str) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let icon = DesktopIcon::new(id, position, label);

        self.desktop_icons.push(icon);

        id
    }

    fn get_focused_window_mut(&mut self) -> Option<&mut Window> {
        self.windows.iter_mut().max_by_key(|w| w.z_index)
    }

    pub fn get_windows_mut(&mut self) -> &mut Vec<Window> {
    &mut self.windows
}
}
