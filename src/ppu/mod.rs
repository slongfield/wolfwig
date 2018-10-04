///! PPU is the Pixel Processing Unit, which displays the Gameboy Screen.
use mem::model::Memory;
use sdl2::{self, pixels, rect};
use std::time::Duration;

// 16 tiles wide, each 8 pixels wide, with a one-pixel spacer. 4 pixels per pixel
// 16 + 16*8*4 = 528
const MAX_X: u32 = 528;
// 8 tiles tall, each 8 pixels, with a one-pixel spaces. 4 pixels per pixel
const MAX_Y: u32 = 264;

const CYCLE_LEN: usize = 70224;

// LCD Y coordinate, current line being rendered.
const LY: usize = 0xFF44;

const LINE_COUNT: u8 = 154;

// Currently, this just displays the tile data for the background tiles.
pub struct Ppu {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    cycle: usize,
}

impl Ppu {
    pub fn new() -> Ppu {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("Gameboy Tile Viewer", MAX_X, MAX_Y)
            .position_centered()
            .build()
            .unwrap();

        Ppu {
            canvas: window.into_canvas().build().unwrap(),
            cycle: 0,
        }
    }

    pub fn step(&mut self, mem: &mut Memory) {
        // Once every 70224 cycles, render.
        if self.cycle == 0 {
            self.render(mem);
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
            self.canvas.present();
        }
        // Every 456 cycles advance one "line".
        // This is a fake placeholder for now. Need to do more realistic handling of the lines to
        // actually show data. This just gets through the bootloader.
        if self.cycle % 456 == 0 {
            let ly = mem.read(LY);
            mem.write(LY, (ly + 1) % LINE_COUNT);
        }
        self.cycle = (self.cycle + 1) % CYCLE_LEN;
    }

    fn render(&mut self, mem: &mut Memory) {
        // Which of the four Y bits are being rendered?
        let mut y_spirte_pos = 0;

        // Which of the 16 possible X tiles are being rendered?
        let mut x_tile_pos = 0;
        // Which of the 8 possible Y tiles are being rendered?
        let mut y_tile_pos = 0;

        // Render the background tileset
        for addr in (0x8000..0x8a00).step_by(2) {
            let upper_byte = mem.read(addr);
            let lower_byte = mem.read(addr + 1);
            for (index, pixel) in (0..8).rev().enumerate() {
                let pixel = (((upper_byte >> pixel) & 1) << 1) | (lower_byte >> pixel);
                let pcolor = pixel * u8::from(85);
                self.canvas
                    .set_draw_color(pixels::Color::RGB(pcolor, pcolor, pcolor));
                let x_pos = x_tile_pos * 8 * 4 + x_tile_pos + index * 4;
                let y_pos = y_tile_pos * 8 * 4 + y_tile_pos + y_spirte_pos * 4;
                self.canvas
                    .fill_rect(rect::Rect::new(x_pos as i32, y_pos as i32, 4, 4));
            }
            y_spirte_pos = (y_spirte_pos + 1) % 8;
            if (y_spirte_pos) == 0 {
                x_tile_pos = (x_tile_pos + 1) % 16;
                if x_tile_pos == 0 {
                    y_tile_pos += 1;
                }
            }
        }
    }
}
