use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::WebColors;

#[derive(Debug, Clone, Copy)]
pub struct WindowStyle {
    pub border_color: Rgb888,
    pub border_width: u32,
    pub titlebar_height: u32,
    pub titlebar_color: Rgb888,
    pub title_text_color: Rgb888,
}

impl Default for WindowStyle {
    fn default() -> Self {
        Self {
            border_color: Rgb888::CSS_DARK_GRAY,
            border_width: 2,
            titlebar_height: 24,
            titlebar_color: Rgb888::CSS_SKY_BLUE,
            title_text_color: Rgb888::CSS_WHITE,
        }
    }
}
