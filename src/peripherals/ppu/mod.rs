use sdl2;
///! PPU is the Pixel Processing Unit, which displays the Gameboy Screen.
use std::thread;
use std::time::Duration;

mod display;
mod fake_display;
mod sdl_display;

const CYCLE_LEN: usize = 70224;

const LINE_COUNT: u8 = 154;

// Written to by 0xFF40.
struct LCDControl {
    enable: bool,
    window_tile_map: bool,
    window_display: bool,
    bg_tile_set: bool,
    bg_tile_map: bool,
    sprite_size: bool,
    sprite_enable: bool,
    bg_enable: bool,
}

impl LCDControl {
    fn new() -> Self {
        Self {
            enable: false,
            window_tile_map: false,
            window_display: false,
            bg_tile_set: false,
            bg_tile_map: false,
            sprite_size: false,
            sprite_enable: false,
            bg_enable: false,
        }
    }

    fn write(&mut self, val: u8) {
        self.enable = val & (1 << 7) != 0;
        self.window_tile_map = val & (1 << 6) != 0;
        self.window_display = val & (1 << 5) != 0;
        self.bg_tile_set = val & (1 << 4) != 0;
        self.bg_tile_map = val & (1 << 3) != 0;
        self.sprite_size = val & (1 << 2) != 0;
        self.sprite_enable = val & (1 << 1) != 0;
        self.bg_enable = val & 1 != 0;
    }

    fn read(&self) -> u8 {
        let mut out = 0;
        if self.enable {
            out |= 1 << 7;
        }
        if self.window_tile_map {
            out |= 1 << 6;
        }
        if self.window_display {
            out |= 1 << 5;
        }
        if self.bg_tile_set {
            out |= 1 << 4;
        }
        if self.bg_tile_map {
            out |= 1 << 3;
        }
        if self.sprite_size {
            out |= 1 << 2;
        }
        if self.sprite_enable {
            out |= 1 << 1;
        }
        if self.bg_enable {
            out |= 1;
        }
        out
    }
}

enum Mode {
    // HBlank
    Mode0,
    // VBlank
    Mode1,
    // Read from OAM, set up sprite info for this line.
    Mode2,
    // Render the line, reads OAM and VRAM.
    Mode3,
}

struct LCDStatus {
    lyc_interrupt: bool,
    mode2_interrupt: bool,
    mode1_interrupt: bool,
    mode0_interrupt: bool,
    lyc_eq_ly: bool,
    mode: Mode,
}

impl LCDStatus {
    fn new() -> Self {
        Self {
            lyc_interrupt: false,
            mode2_interrupt: false,
            mode1_interrupt: false,
            mode0_interrupt: false,
            lyc_eq_ly: false,
            mode: Mode::Mode2,
        }
    }
    fn write(&mut self, val: u8) {
        self.lyc_interrupt = val & (1 << 6) != 0;
        self.mode2_interrupt = val & (1 << 5) != 0;
        self.mode1_interrupt = val & (1 << 4) != 0;
        self.mode0_interrupt = val & (1 << 3) != 0;
    }
    fn read(&self, lcd_y_eq_compare: bool) -> u8 {
        let mut out = 1 << 7;
        if self.lyc_interrupt {
            out |= 1 << 6;
        }
        if self.mode2_interrupt {
            out |= 1 << 5;
        }
        if self.mode1_interrupt {
            out |= 1 << 4;
        }
        if self.mode0_interrupt {
            out |= 1 << 3;
        }
        if lcd_y_eq_compare {
            out |= 1 << 2;
        }
        match self.mode {
            Mode::Mode0 => out |= 0,
            Mode::Mode1 => out |= 1,
            Mode::Mode2 => out |= 2,
            Mode::Mode3 => out |= 3,
        }
        out
    }
}

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
    control: LCDControl,
    status: LCDStatus,
    scroll_x: u8,
    scroll_y: u8,
    lcd_y: u8,
    lcd_y_compare: u8,
    dma: u8,
    rendered: bool,
}

impl Ppu {
    // LCD control
    const LCDC: u16 = 0xFF40;
    // LCD status register (read/write bits 2-6, RO bits 0-1)
    const STAT: u16 = 0xFF41;
    // Scroll Y/X: Specifies the position of the background window. Changes take effect at end of
    // current scanline.
    const SCY: u16 = 0xFF42;
    const SCX: u16 = 0xFF43;
    // LCD Y coordinate, current line being rendered. Read-only.
    const LY: u16 = 0xFF44;
    // LCD Y compare. When equal to LY, bit in STAT is set, and (if enabled), STAT interrupt
    // fires.
    const LYC: u16 = 0xFF45;
    // Writes to this register starts a DMA transfer from the address written to OAM.
    const DMA: u16 = 0xFF46;
    // Background palette data
    const BGP: u16 = 0xFF47;
    // Object Palette 0 Data
    const OBP0: u16 = 0xFF48;
    // Object Palette 1 Data
    const OBP1: u16 = 0xFF49;
    // Window Y and X position. This is an alternate background that is displayed above the
    // current background if visible.
    const WY: u16 = 0xFF4A;
    const WX: u16 = 0xFF4B;

    pub fn new_sdl(video_subsystem: sdl2::VideoSubsystem) -> Self {
        Self {
            display: Box::new(sdl_display::SdlDisplay::new(video_subsystem)),
            cycle: 0,
            wait_for_frame: false,
            vram: [0; 0x2000],
            oam: [0; 0x100],
            lcd_y: 0,
            scroll_x: 0,
            scroll_y: 0,
            lcd_y_compare: 0,
            dma: 0,
            rendered: false,
            control: LCDControl::new(),
            status: LCDStatus::new(),
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
            scroll_x: 0,
            scroll_y: 0,
            lcd_y_compare: 0,
            dma: 0,
            rendered: false,
            control: LCDControl::new(),
            status: LCDStatus::new(),
        }
    }

    pub fn step(&mut self) {
        // Once every 70224 cycles, render.
        if self.cycle == 0 {
            self.rendered = false;
        }
        if self.lcd_y == 100 && !self.rendered {
            self.render();
            if self.wait_for_frame {
                thread::sleep(Duration::new(0, 1_000_000_000_u32 / 60));
            }
            self.display.show();
            self.rendered = true;
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
            addr @ 0x8000..=0x9FFF => match self.status.mode {
                Mode::Mode0 | Mode::Mode1 | Mode::Mode2 => {
                    if let Some(old) = self.vram.get_mut((addr as usize) - 0x8000) {
                        *old = val;
                    }
                }
                Mode::Mode3 => {}
            },
            addr @ 0xFE00..=0xFE9F => match self.status.mode {
                Mode::Mode0 | Mode::Mode1 => {
                    if let Some(old) = self.oam.get_mut((addr as usize) - 0xFE00) {
                        *old = val;
                    }
                }
                Mode::Mode2 | Mode::Mode3 => {}
            },
            Self::LCDC => self.control.write(val),
            Self::STAT => self.status.write(val),
            Self::LY => {}
            Self::LYC => self.lcd_y_compare = val,
            0xFF40..=0xFF4B => info!(
                "Attempted to write to unhandled PPU register: {:#04X}",
                address
            ),
            addr => panic!("Attempted to write PPU with unmapped addr: {:#x}", addr),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            addr @ 0x8000..=0x9FFF => match self.status.mode {
                Mode::Mode0 | Mode::Mode1 | Mode::Mode2 => {
                    if let Some(val) = self.vram.get((addr as usize) - 0x8000) {
                        *val
                    } else {
                        error!("Read from unmapped VRAM address {:#04X}", addr);
                        0
                    }
                }
                Mode::Mode3 => 0xFF,
            },
            addr @ 0xFE00..=0xFE9F => match self.status.mode {
                Mode::Mode0 | Mode::Mode1 => {
                    if let Some(val) = self.oam.get((addr as usize) - 0xFE00) {
                        *val
                    } else {
                        error!("Read from unmapped OAM address {:#04X}", addr);
                        0
                    }
                }
                Mode::Mode2 | Mode::Mode3 => 0xFF,
            },
            Self::LCDC => self.control.read(),
            Self::STAT => self.status.read(self.lcd_y == self.lcd_y_compare),
            Self::LY => self.lcd_y,
            Self::LYC => self.lcd_y_compare,
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
