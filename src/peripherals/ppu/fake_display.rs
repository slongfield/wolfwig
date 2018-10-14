///! Fake display for testing.
use peripherals::ppu::display;

pub struct FakeDisplay {}

impl FakeDisplay {
    pub fn new() -> Self {
        Self {}
    }
}

impl display::Display for FakeDisplay {
    fn clear(&mut self, color: display::Color) {}
    fn draw_pixel(&mut self, x: usize, y: usize, color: display::Color) -> Result<(), String> {
        Ok(())
    }
    fn show(&mut self) {}
}
