///! PPU is the Pixel Processing Unit, which displays the Gameboy Screen.
use sdl2::{self, pixels, rect};
use std::thread;
use std::time::Duration;

// 16 tiles wide, each 8 pixels wide, with a one-pixel spacer. 4 pixels per pixel
// 16 + 16*8*4 = 528
const MAX_X: u32 = 528;
// 8 tiles tall, each 8 pixels, with a one-pixel spaces. 4 pixels per pixel
const MAX_Y: u32 = 528;

const CYCLE_LEN: usize = 70224;

const LINE_COUNT: u8 = 154;

// Currently, this just displays the tile data for the background tiles.
pub struct Ppu {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    cycle: usize,
    wait_for_frame: bool,
    // Video RAM. TODO(slongfield): In CGB, should be switchable banks.
    // Ox8000-0x9FFF
    vram: [u8; 0x2000],
    // Sprite attribute table.
    // 0xFE00-0xFE9F
    oam: [u8; 0x100],
    // I/O registers
    lcd_y: u8,
}

impl Ppu {
    // LCD Y coordinate, current line being rendered.
    const LY: u16 = 0xFF44;

    pub fn new(video_subsystem: sdl2::VideoSubsystem) -> Self {
        let window = video_subsystem
            .window("Gameboy Tile Viewer", MAX_X, MAX_Y)
            .position_centered()
            .build()
            .unwrap();

        Self {
            canvas: window.into_canvas().build().unwrap(),
            cycle: 0,
            wait_for_frame: false,
            vram: [0; 0x2000],
            oam: [0; 0x100],
            lcd_y: 0,
        }
    }

    pub fn step(&mut self) {
        // Once every 70224 cycles, render.
        if self.cycle == 0 {
            self.render();
            if self.wait_for_frame {
                thread::sleep(Duration::new(0, 1_000_000_000_u32 / 60));
            }
            self.canvas.present();
        }
        // Every 456 cycles advance one "line".
        // This is a fake placeholder for now. Need to do more realistic handling of the lines to
        // actually show data. This just gets through the bootloader.
        if self.cycle % 456 == 0 {
            self.lcd_y = (self.lcd_y + 1) % LINE_COUNT;
        }
        self.cycle = (self.cycle + 1) % CYCLE_LEN;
    }

    pub fn write(&mut self, address: u16, val: u8) {
        match address {
            addr @ 0x8000..=0x9FFF => {
                if let Some(old) = self.vram.get_mut((addr as usize) - 0x8000) {
                    *old = val;
                }
            }
            addr @ 0xFE00..=0xFE9F => {
                if let Some(old) = self.oam.get_mut((addr as usize) - 0xFE00) {
                    *old = val;
                }
            }
            Self::LY => self.lcd_y = val,
            0xFF40..=0xFF4B => info!(
                "Attempted to write to unhandled PPU register: {:#04X}",
                address
            ),
            addr => panic!("Attempted to write PPU with unmapped addr: {:#x}", addr),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            addr @ 0x8000..=0x9FFF => {
                if let Some(val) = self.vram.get((addr as usize) - 0x8000) {
                    *val
                } else {
                    error!("Read from unmapped VRAM address {:#04X}", addr);
                    0
                }
            }
            addr @ 0xFE00..=0xFE9F => {
                if let Some(val) = self.oam.get((addr as usize) - 0xFE00) {
                    *val
                } else {
                    error!("Read from unmapped OAM address {:#04X}", addr);
                    0
                }
            }
            Self::LY => self.lcd_y,
            0xFF40..=0xFF4B => {
                info!(
                    "Attempted to read from unhandled PPU register: {:#04X}",
                    address
                );
                0
            }
            addr => panic!("Attempted to write PPU with unmapped addr: {:#x}", addr),
        }
    }

    fn render(&mut self) {
        // Which of the four Y bits are being rendered?
        let mut y_spirte_pos: i32 = 0;

        // Which of the 16 possible X tiles are being rendered?
        let mut x_tile_pos: i32 = 0;
        // Which of the 8 possible Y tiles are being rendered?
        let mut y_tile_pos: i32 = 0;

        self.canvas.set_draw_color(pixels::Color::RGB(255, 0, 0));
        self.canvas.clear();

        // Render the backgr und tileset
        for addr in (0..0x1000).step_by(2) {
            let upper_byte = self.vram[addr];
            let lower_byte = self.vram[addr + 1];
            for (index, pixel) in (0..8).rev().enumerate() {
                let index = index as i32;
                let pixel = (((upper_byte >> pixel) & 1) << 1) | ((lower_byte >> pixel) & 1);
                let pcolor = pixel.wrapping_mul(84);
                self.canvas
                    .set_draw_color(pixels::Color::RGB(pcolor, pcolor, pcolor));
                let x_pos = x_tile_pos * 8 * 4 + x_tile_pos + index * 4;
                let y_pos = y_tile_pos * 8 * 4 + y_tile_pos + y_spirte_pos * 4;
                self.canvas
                    .fill_rect(rect::Rect::new(x_pos, y_pos, 4, 4))
                    .expect("Could not draw rect");
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
