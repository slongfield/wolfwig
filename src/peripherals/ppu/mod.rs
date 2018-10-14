use sdl2;
///! PPU is the Pixel Processing Unit, which displays the Gameboy Screen.
use std::thread;
use std::time::Duration;

mod display;
mod fake_display;
mod sdl_display;

const CYCLE_LEN: usize = 70224;

const LINE_COUNT: u8 = 154;

// Currently, this just displays the tile data for the background tiles.
pub struct Ppu {
    display: Box<display::Display>,
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

    pub fn new_sdl(video_subsystem: sdl2::VideoSubsystem) -> Self {
        Self {
            display: Box::new(sdl_display::SdlDisplay::new(video_subsystem)),
            cycle: 0,
            wait_for_frame: false,
            vram: [0; 0x2000],
            oam: [0; 0x100],
            lcd_y: 0,
        }
    }

    pub fn new_fake() -> Self {
        Self {
            display: Box::new(fake_display::FakeDisplay::new()),
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
            self.display.show();
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
        let mut y_spirte_pos: usize = 0;
        let mut x_tile_pos: usize = 0;
        let mut y_tile_pos: usize = 0;

        self.display.clear(display::Color::RGB(255, 0, 0));

        for addr in (0..0x1000).step_by(2) {
            let upper_byte = self.vram[addr];
            let lower_byte = self.vram[addr + 1];
            for (index, pixel) in (0..8).rev().enumerate() {
                let pixel = (((upper_byte >> pixel) & 1) << 1) | ((lower_byte >> pixel) & 1);
                let pcolor = pixel.wrapping_mul(84);
                let x = x_tile_pos * 8 + x_tile_pos + index;
                let y = y_tile_pos * 8 + y_tile_pos + y_spirte_pos;
                self.display
                    .draw_pixel(x, y, display::Color::RGB(pcolor, pcolor, pcolor))
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
