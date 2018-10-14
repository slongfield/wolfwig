///! Interface that needs to be implemented to create a Display.
use std::result::Result;

pub enum Color {
    Black,
    RGB(u8, u8, u8),
}

pub trait Display {
    fn clear(&mut self, color: Color);
    fn draw_pixel(&mut self, x: usize, y: usize, color: Color) -> Result<(), String>;
    fn show(&mut self);
}
