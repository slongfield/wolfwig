use peripherals::interrupt::{Interrupt, Irq};
use peripherals::Dma;
use sdl2;
use std::thread;
use std::time::{Duration, Instant};

mod display;
mod fake_display;
mod sdl_display;

const CYCLE_LEN: usize = 70224;

const LINE_COUNT: u8 = 154;
const VISIBLE_COUNT: u8 = 144;
const MODE0_CYCLES: u8 = 51;
const MODE1_CYCLES: u8 = 114; // cycles per line
const MODE2_CYCLES: u8 = 20;
const MODE3_CYCLES: u8 = 43;

// Written to by 0xFF40
bitflags! {
    struct LCDControl: u8 {
        const Enable =        0b1000_0000;
        const WindowTileMap = 0b0100_0000;
        const WindowDisplay = 0b0010_0000;
        const BgTileSet =     0b0001_0000;
        const BgTileMap =     0b0000_1000;
        const SpriteSize =    0b0000_0100;
        const SpriteEnable =  0b0000_0010;
        const BgEnable =      0b0000_0001;
    }
}

impl LCDControl {
    fn new() -> Self {
        Self::empty()
    }

    fn write(&mut self, val: u8) {
        self.remove(Self::all());
        self.insert(Self::from_bits_truncate(val));
    }

    fn read(&self) -> u8 {
        self.bits()
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
            mode: Mode::Mode0,
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

#[derive(Debug)]
struct Sprite {
    y: u8,
    x: u8,
    tile: u8,
    flags: u8,
}

impl Sprite {
    fn new(y: u8, x: u8, tile: u8, flags: u8) -> Self {
        Self { y, x, tile, flags }
    }

    // TODO(slongfield): Implement accessors for the other flags.
    fn palette(&self) {
        self.flags & (1 << 4) != 0;
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
    bg_palette: u8,
    mode_cycle: u8,
    sprites: Vec<Sprite>,
    before: Instant,
    dma: Dma,
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

    // Number of microseconds between frames.
    const INTERVAL: u64 = 16_666;

    pub fn new_sdl(video_subsystem: sdl2::VideoSubsystem) -> Self {
        Self {
            display: Box::new(sdl_display::SdlDisplay::new(video_subsystem)),
            cycle: 0,
            wait_for_frame: true,
            vram: [0; 0x2000],
            oam: [0; 0x100],
            lcd_y: 0,
            scroll_x: 0,
            scroll_y: 0,
            lcd_y_compare: 0,
            control: LCDControl::new(),
            status: LCDStatus::new(),
            bg_palette: 0,
            mode_cycle: 0,
            sprites: vec![],
            before: Instant::now(),
            dma: Dma::new(),
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
            control: LCDControl::new(),
            status: LCDStatus::new(),
            bg_palette: 0,
            mode_cycle: 0,
            sprites: vec![],
            before: Instant::now(),
            dma: Dma::new(),
        }
    }

    pub fn step(&mut self, interrupt: &mut Interrupt, dma: &mut Dma) {
        if self.control.contains(LCDControl::Enable) {
            match self.status.mode {
                Mode::Mode0 => self.mode0(interrupt),
                Mode::Mode1 => self.mode1(interrupt),
                Mode::Mode2 => self.mode2(interrupt),
                Mode::Mode3 => self.mode3(interrupt),
            }
        }
        if dma.enabled {
            dma.source += 4;
            dma.dest += 4;
            if dma.dest > 0xFE9F {
                dma.enabled = false;
            }
        }
        if self.dma.enabled {
            dma.enabled = true;
            dma.source = self.dma.source;
            dma.dest = self.dma.dest;
            self.dma = Dma::new();
        }
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
            // TODO(slongfield): Figure out when SCY and SCX are writeable.
            Self::SCY => self.scroll_y = val,
            Self::SCX => self.scroll_x = val,
            Self::LY => {}
            Self::LYC => self.lcd_y_compare = val,
            Self::DMA => {
                self.dma.enabled = true;
                self.dma.source = u16::from(val) * 0x100;
                self.dma.dest = 0xFE00;
            }
            Self::BGP => self.bg_palette = val,
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
            Self::SCY => self.scroll_y,
            Self::SCX => self.scroll_x,
            Self::LY => self.lcd_y,
            Self::LYC => self.lcd_y_compare,
            Self::BGP => self.bg_palette,
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

    // HBlank, do nothing.
    fn mode0(&mut self, interrupt: &mut Interrupt) {
        self.mode_cycle += 1;
        if self.mode_cycle == MODE0_CYCLES {
            self.lcd_y += 1;
            self.update_ly_interrupt(interrupt);
            self.mode_cycle = 0;
            if self.lcd_y == VISIBLE_COUNT {
                self.status.mode = Mode::Mode1;
            } else {
                self.status.mode = Mode::Mode2;
            }
            self.update_mode_interrupt(interrupt);
        }
    }

    // VBlank, do nothing
    fn mode1(&mut self, interrupt: &mut Interrupt) {
        self.mode_cycle += 1;
        if self.mode_cycle == MODE1_CYCLES {
            self.lcd_y += 1;
            self.update_ly_interrupt(interrupt);
            self.mode_cycle = 0;
            // TODO(slongfield): Compare LCD Y
            if self.lcd_y == LINE_COUNT {
                self.lcd_y = 0;
                self.status.mode = Mode::Mode2;
                self.update_mode_interrupt(interrupt);

                self.display.show();
                if self.wait_for_frame {
                    let now = Instant::now();
                    let dt = u64::from(now.duration_since(self.before).subsec_micros());
                    if dt < Self::INTERVAL {
                        thread::sleep(Duration::from_micros(Self::INTERVAL - dt));
                    }
                    self.before = now;
                }
            }
        }
    }

    // OAM read, build sprite list.
    fn mode2(&mut self, interrupt: &mut Interrupt) {
        if self.mode_cycle == 0 {
            self.sprites = vec![];
            for entry in self.oam.chunks(4) {
                let y = *entry.get(0).unwrap_or(&0);
                let x = *entry.get(1).unwrap_or(&0);
                let tile = *entry.get(2).unwrap_or(&0);
                let flags = *entry.get(3).unwrap_or(&0);
                // Only add the sprite if it'll be visibile.
                // TODO(slongfield): Handle double-tall sprites.
                if self.lcd_y + 8 < y && self.lcd_y + 16 >= y {
                    self.sprites.push(Sprite::new(y, x, tile, flags));
                }
            }
            // Reverse sort by X, since smallest X gets highest priority, so want to draw it
            // last.
            self.sprites.sort_unstable_by(|a, b| (b.x).cmp(&a.x));
        }
        self.mode_cycle += 1;
        if self.mode_cycle == MODE2_CYCLES {
            self.mode_cycle = 0;
            self.status.mode = Mode::Mode3;
            self.update_mode_interrupt(interrupt);
        }
    }

    // Draw mode!
    fn mode3(&mut self, interrupt: &mut Interrupt) {
        // Only draw every other cycle, since we're drawing 8 pixels per cycle, but have 40 cycles
        // to draw 160 pixels.
        // TODO(slongfield): Model pixel fifo
        if self.mode_cycle % 2 == 0 {
            let mut pixels: [u8; 8] = [0; 8];
            let y = u16::from(self.scroll_y.wrapping_add(self.lcd_y));
            let x = u16::from(self.scroll_x.wrapping_add(self.mode_cycle * 4));
            let y_tile = y / 8;
            let x_tile = x / 8;
            // Get background pixels.
            {
                let tile_map_start: u16 = if self.control.contains(LCDControl::BgTileMap) {
                    0x1C00
                } else {
                    0x1800
                };
                let tile = self
                    .vram
                    .get((tile_map_start + y_tile * 32 + x_tile) as usize)
                    .unwrap_or(&0);
                let bg_tileset_start = if self.control.contains(LCDControl::BgTileSet) {
                    0x0
                } else {
                    0x800
                };
                let addr = usize::from(bg_tileset_start + u16::from(*tile) * 16 + (y % 8) * 2);
                let upper_byte = self.vram[addr];
                let lower_byte = self.vram[addr + 1];
                for (index, pixel) in (0..8).rev().enumerate() {
                    let pixel = (((upper_byte >> pixel) & 1) << 1) | ((lower_byte >> pixel) & 1);
                    pixels[index] = self.bg_color(pixel);
                }
            }
            // TODO(slongfield): Get window pixels.
            if self.control.contains(LCDControl::SpriteEnable) {
                // TODO(slongfield): Need to handle background pixel pallete data later, since
                // background 00 should draw over sprite pixels. Thankfully can ignore priority
                // for Tetris.
                let x = self.mode_cycle * 4;
                for sprite in self.sprites.iter() {
                    // TODO(slongfield): Handle inverted sprites and double-tall sprites.
                    let tile_y = 7 - u16::from((sprite.y - self.lcd_y + 15) % 8);
                    if x + 8 <= sprite.x && x + 16 > sprite.x {
                        let addr = usize::from(u16::from(sprite.tile) * 16 + tile_y * 2);
                        let upper_byte = self.vram[addr];
                        let lower_byte = self.vram[addr + 1];
                        for (index, pixel) in (0..8).rev().enumerate() {
                            let pixel =
                                (((upper_byte >> pixel) & 1) << 1) | ((lower_byte >> pixel) & 1);
                            pixels[index] = self.bg_color(pixel);
                        }
                    }
                }
            }
            for (index, pixel) in pixels.iter().enumerate() {
                // TODO(slongfield): Adjust to taste.
                let color = match pixel {
                    0b00 => display::Color::RGB(155, 188, 15),
                    0b01 => display::Color::RGB(48, 98, 48),
                    0b10 => display::Color::RGB(139, 172, 15),
                    _ => display::Color::RGB(15, 56, 15),
                };
                let x = (self.mode_cycle) * 4 + (index as u8);
                self.display
                    .draw_pixel(x as usize, self.lcd_y as usize, color)
                    .expect("Could not draw rectangle");
            }
        }
        self.mode_cycle += 1;
        if self.mode_cycle == MODE3_CYCLES {
            self.mode_cycle = 0;
            self.status.mode = Mode::Mode0;
        }
    }

    fn update_ly_interrupt(&mut self, interrupt: &mut Interrupt) {
        if self.lcd_y == self.lcd_y_compare {
            self.status.lyc_eq_ly = true;
        } else {
            self.status.lyc_eq_ly = false;
        }
        if self.status.lyc_interrupt {
            interrupt.set_flag(Irq::LCDStat, true);
        }
    }

    fn update_mode_interrupt(&mut self, interrupt: &mut Interrupt) {
        match self.status.mode {
            Mode::Mode0 => {
                if self.status.mode0_interrupt {
                    interrupt.set_flag(Irq::LCDStat, true);
                }
            }
            Mode::Mode1 => {
                if self.status.mode1_interrupt {
                    interrupt.set_flag(Irq::LCDStat, true);
                }
                interrupt.set_flag(Irq::Vblank, true);
            }
            Mode::Mode2 => {
                if self.status.mode2_interrupt {
                    interrupt.set_flag(Irq::LCDStat, true);
                }
            }
            Mode::Mode3 => {}
        }
    }

    fn bg_color(&self, index: u8) -> u8 {
        let shade = match index {
            0b00 => self.bg_palette,
            0b01 => self.bg_palette >> 2,
            0b10 => self.bg_palette >> 4,
            _ => self.bg_palette >> 6,
        };
        shade & 0x3
    }
}
