use super::AppInterface;
use crate::window::{EmbeddedGraphicsBuffer, WindowEvent};
use core::convert::Infallible;
use embedded_graphics::Pixel;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use oorandom;
use virtio_input_decoder::Key;

// 重用gui_snake.rs中的大部分代码，但修改为适应窗口系统

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
    None,
}

struct Snake<T: PixelColor, const MAX_SIZE: usize> {
    parts: [Pixel<T>; MAX_SIZE],
    len: usize,
    direction: Direction,
    size_x: u32,
    size_y: u32,
}

struct SnakeIntoIterator<'a, T: PixelColor, const MAX_SIZE: usize> {
    snake: &'a Snake<T, MAX_SIZE>,
    index: usize,
}

impl<'a, T: PixelColor, const MAX_SIZE: usize> IntoIterator for &'a Snake<T, MAX_SIZE> {
    type Item = Pixel<T>;
    type IntoIter = SnakeIntoIterator<'a, T, MAX_SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        SnakeIntoIterator {
            snake: self,
            index: 0,
        }
    }
}

impl<'a, T: PixelColor, const MAX_SIZE: usize> Iterator for SnakeIntoIterator<'a, T, MAX_SIZE> {
    type Item = Pixel<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.snake.len {
            let cur = self.snake.parts[self.index];
            self.index += 1;
            return Some(cur);
        }
        None
    }
}

impl<T: PixelColor, const MAX_SIZE: usize> Snake<T, MAX_SIZE> {
    fn new(color: T, size_x: u32, size_y: u32) -> Self {
        Self {
            parts: [Pixel::<T>(Point { x: 0, y: 0 }, color); MAX_SIZE],
            len: 1,
            direction: Direction::None,
            size_x,
            size_y,
        }
    }

    fn set_direction(&mut self, direction: Direction) {
        // 防止直接反向移动
        match (self.direction, direction) {
            (Direction::Left, Direction::Right)
            | (Direction::Right, Direction::Left)
            | (Direction::Up, Direction::Down)
            | (Direction::Down, Direction::Up) => return,
            _ => self.direction = direction,
        }
    }

    fn contains(&self, this: Point) -> bool {
        for part in self.into_iter() {
            if part.0 == this {
                return true;
            };
        }
        false
    }

    fn grow(&mut self) {
        if self.len < MAX_SIZE - 1 {
            self.len += 1;
        }
    }

    fn make_step(&mut self) {
        let mut i = self.len;
        while i > 0 {
            self.parts[i] = self.parts[i - 1];
            i -= 1;
        }

        match self.direction {
            Direction::Left => {
                if self.parts[0].0.x == 0 {
                    self.parts[0].0.x = (self.size_x - 1) as i32;
                } else {
                    self.parts[0].0.x -= 1;
                }
            }
            Direction::Right => {
                if self.parts[0].0.x == (self.size_x - 1) as i32 {
                    self.parts[0].0.x = 0;
                } else {
                    self.parts[0].0.x += 1;
                }
            }
            Direction::Up => {
                if self.parts[0].0.y == 0 {
                    self.parts[0].0.y = (self.size_y - 1) as i32;
                } else {
                    self.parts[0].0.y -= 1;
                }
            }
            Direction::Down => {
                if self.parts[0].0.y == (self.size_y - 1) as i32 {
                    self.parts[0].0.y = 0;
                } else {
                    self.parts[0].0.y += 1;
                }
            }
            Direction::None => {}
        }
    }
}

struct Food<T: PixelColor> {
    size_x: u32,
    size_y: u32,
    place: Pixel<T>,
    rng: oorandom::Rand32,
}

impl<T: PixelColor> Food<T> {
    pub fn new(color: T, size_x: u32, size_y: u32) -> Self {
        let seed = 4;
        let rng = oorandom::Rand32::new(seed);
        Food {
            size_x,
            size_y,
            place: Pixel(Point { x: 0, y: 0 }, color),
            rng,
        }
    }

    fn replace<'a, const MAX_SIZE: usize>(&mut self, iter_source: &Snake<T, MAX_SIZE>) {
        let mut p: Point;
        'outer: loop {
            let random_number = self.rng.rand_u32();
            let blocked_positions = iter_source.into_iter();
            p = Point {
                x: ((random_number >> 24) as u16 % self.size_x as u16) as i32,
                y: ((random_number >> 16) as u16 % self.size_y as u16) as i32,
            };
            for blocked_position in blocked_positions {
                if p == blocked_position.0 {
                    continue 'outer;
                }
            }
            break;
        }
        self.place = Pixel::<T> {
            0: p,
            1: self.place.1,
        }
    }

    fn get_pixel(&self) -> Pixel<T> {
        self.place
    }
}

struct SnakeGame<const MAX_SNAKE_SIZE: usize, T: PixelColor> {
    snake: Snake<T, MAX_SNAKE_SIZE>,
    food: Food<T>,
    food_age: u32,
    food_lifetime: u32,
    size_x: u32,
    size_y: u32,
    scale_x: u32,
    scale_y: u32,
    game_over: bool,
    score: u32,
}

impl<const MAX_SIZE: usize, T: PixelColor> SnakeGame<MAX_SIZE, T> {
    pub fn new(
        size_x: u32,
        size_y: u32,
        scale_x: u32,
        scale_y: u32,
        snake_color: T,
        food_color: T,
        food_lifetime: u32,
    ) -> Self {
        let snake = Snake::<T, MAX_SIZE>::new(snake_color, size_x / scale_x, size_y / scale_y);
        let mut food = Food::<T>::new(food_color, size_x / scale_x, size_y / scale_y);
        food.replace(&snake);

        SnakeGame {
            snake,
            food,
            food_age: 0,
            food_lifetime,
            size_x,
            size_y,
            scale_x,
            scale_y,
            game_over: false,
            score: 0,
        }
    }

    pub fn set_direction(&mut self, direction: Direction) {
        if !self.game_over {
            self.snake.set_direction(direction);
        }
    }

    pub fn update(&mut self) -> bool {
        if self.game_over {
            return false;
        }

        self.snake.make_step();

        // 检查蛇是否撞到自己
        let head = self.snake.parts[0].0;
        for i in 1..self.snake.len {
            if head == self.snake.parts[i].0 {
                self.game_over = true;
                return true;
            }
        }

        // 检查是否吃到食物
        let hit = self.snake.contains(self.food.get_pixel().0);
        if hit {
            self.snake.grow();
            self.score += 1;
        }

        self.food_age += 1;
        if self.food_age >= self.food_lifetime || hit {
            self.food.replace(&self.snake);
            self.food_age = 0;
        }

        true
    }

    pub fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = T>,
    {
        // 绘制每个蛇身体部分
        for part in self.snake.into_iter() {
            Rectangle::new(
                Point::new(
                    part.0.x * self.scale_x as i32,
                    part.0.y * self.scale_y as i32,
                ),
                Size::new(self.scale_x as u32, self.scale_y as u32),
            )
            .into_styled(PrimitiveStyle::with_fill(part.1))
            .draw(target)?;
        }

        // 绘制食物
        let food = self.food.get_pixel();
        Rectangle::new(
            Point::new(
                food.0.x * self.scale_x as i32,
                food.0.y * self.scale_y as i32,
            ),
            Size::new(self.scale_x as u32, self.scale_y as u32),
        )
        .into_styled(PrimitiveStyle::with_fill(food.1))
        .draw(target)?;

        Ok(())
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    #[allow(unused)]
    pub fn get_score(&self) -> u32 {
        self.score
    }

    pub fn reset(&mut self) {
        let snake = Snake::<T, MAX_SIZE>::new(
            self.snake.parts[0].1,
            self.size_x / self.scale_x,
            self.size_y / self.scale_y,
        );
        self.snake = snake;
        self.food.replace(&self.snake);
        self.food_age = 0;
        self.game_over = false;
        self.score = 0;
    }
}

// 实现应用接口
pub struct SnakeApp {
    game: Option<SnakeGame<20, Rgb888>>,
    size: Size,
    #[allow(unused)]
    last_update: u64,
}

impl SnakeApp {
    pub fn new() -> Self {
        Self {
            game: None,
            size: Size::new(0, 0),
            last_update: 0,
        }
    }
}

impl AppInterface for SnakeApp {
    fn init(&mut self, size: Size) -> &str {
        self.size = size;

        // 创建游戏实例，调整缩放因子使其适应窗口大小
        let cell_size = 15; // 每个蛇身体格子的大小
        self.game = Some(SnakeGame::<20, Rgb888>::new(
            size.width,
            size.height,
            cell_size,
            cell_size,
            Rgb888::RED,
            Rgb888::YELLOW,
            200,
        ));

        "Snake Game"
    }

    fn handle_event(&mut self, event: WindowEvent) {
        if let Some(game) = &mut self.game {
            match event {
                WindowEvent::KeyPress(key) => {
                    match key {
                        Key::W => {
                            println!("key w presed");
                            game.set_direction(Direction::Up)
                        }
                        Key::S => {
                            println!("key s presed");
                            game.set_direction(Direction::Down)
                        }
                        Key::A => {
                            println!("key a presed");
                            game.set_direction(Direction::Left)
                        }
                        Key::D => {
                            println!("key d presed");
                            game.set_direction(Direction::Right)
                        }
                        Key::R => game.reset(), // 重置游戏
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    fn render(&mut self, buffer: &mut EmbeddedGraphicsBuffer) -> Result<(), Infallible> {
        // 清空缓冲区
        buffer.clear(Rgb888::BLACK)?;

        if let Some(game) = &self.game {
            // 绘制游戏
            game.draw(buffer)?;

            // 如果游戏结束，显示提示信息
            if game.is_game_over() {
                // 在这里可以添加绘制游戏结束文字的代码
                // 简单起见，目前省略
            }
        }

        Ok(())
    }

    fn update(&mut self) -> bool {
        if let Some(game) = &mut self.game {
            game.update()
        } else {
            false
        }
    }

    fn needs_timer(&self) -> bool {
        true
    }

    fn timer_interval(&self) -> u64 {
        100 // 控制游戏速度
    }
}
