///! Fake display for testing.
use peripherals::ppu::display;

pub struct FakeDisplay {}

impl FakeDisplay {
    pub fn new() -> Self {
        Self {}
    }
}

impl display::Display for FakeDisplay {
    fn clear(&mut self, _color: display::Color) {}
    fn draw_pixel(&mut self, _x: usize, _y: usize, _color: display::Color) -> Result<(), String> {
        Ok(())
    }
    fn show(&mut self) {}
}
