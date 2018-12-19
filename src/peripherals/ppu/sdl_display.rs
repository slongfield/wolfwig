use peripherals::ppu::display;
use sdl2::{self, pixels, rect};
use std::result::Result;

// 4 pixels per pixel
const MAX_X: u32 = 640;
const MAX_Y: u32 = 576;

// Should 'Display' trait actaully be 'Window'?
pub struct SdlDisplay {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
}

impl SdlDisplay {
    pub fn new(video_subsystem: sdl2::VideoSubsystem) -> Self {
        let window = video_subsystem
            .window("Wolfwig Gameboy Emulator", MAX_X, MAX_Y)
            .position_centered()
            .build()
            .unwrap();

        Self {
            canvas: window.into_canvas().build().unwrap(),
        }
    }
}

impl display::Display for SdlDisplay {
    fn clear(&mut self, color: display::Color) {
        if let display::Color::RGB(r, g, b) = color {
            self.canvas.set_draw_color(pixels::Color::RGB(r, g, b));
        } else {
            self.canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        }
        self.canvas.clear();
    }

    fn draw_pixel(&mut self, x: usize, y: usize, color: display::Color) -> Result<(), String> {
        if let display::Color::RGB(r, g, b) = color {
            self.canvas.set_draw_color(pixels::Color::RGB(r, g, b));
        } else {
            self.canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        }
        self.canvas
            .fill_rect(rect::Rect::new((x * 4) as i32, (y * 4) as i32, 4, 4))
    }

    fn show(&mut self) {
        self.canvas.present();
    }
}
