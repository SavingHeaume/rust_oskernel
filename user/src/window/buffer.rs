use alloc::vec;
use alloc::vec::Vec;
use embedded_graphics::{pixelcolor::Rgb888, prelude::*, primitives::Rectangle};

#[derive(Clone)]
pub struct EmbeddedGraphicsBuffer {
    pub size: Size,
    pub pixels: Vec<Rgb888>, // 存储ARGB格式
}

impl EmbeddedGraphicsBuffer {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            pixels: vec![Rgb888::BLACK; (size.width * size.height) as usize],
        }
    }

    /// 实现块传输优化
    pub fn blit_to<D>(&self, target: &mut D, position: Point) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb888>,
    {
        let width = self.size.width as usize;

        target.draw_iter(
            self.pixels
                .iter()
                .enumerate()
                .filter(|(_, color)| **color != Rgb888::BLACK)
                .map(|(idx, color)| {
                    let x = (idx % width) as i32 + position.x;
                    let y = (idx / width) as i32 + position.y;
                    Pixel(Point::new(x, y), *color)
                }),
        )
    }
}

impl DrawTarget for EmbeddedGraphicsBuffer {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for px in pixels {
            let x = px.0.x;
            let y = px.0.y;

            if x >= 0 && y >= 0 && x < self.size.width as i32 && y < self.size.height as i32 {
                let idx = (y * self.size.width as i32 + x) as usize;
                self.pixels[idx] = px.1;
            }
        }
        Ok(())
    }
}

impl OriginDimensions for EmbeddedGraphicsBuffer {
    fn size(&self) -> Size {
        self.size
    }
}

impl Drawable for EmbeddedGraphicsBuffer {
    type Color = Rgb888;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.blit_to(target, Point::zero())
    }
}
