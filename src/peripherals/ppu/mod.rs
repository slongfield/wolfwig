use peripherals::interrupt::Interrupt;
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

bitflags! {
    pub struct LCDControl: u8 {
        const ENABLE =        0b1000_0000;
        const WINDOW_TILE_MAP = 0b0100_0000;
        const WINDOW_DISPLAY = 0b0010_0000;
        const BG_TILE_SET =     0b0001_0000;
        const BG_TILE_MAP =     0b0000_1000;
        const SPRITE_SIZE =    0b0000_0100;
        const SPRITE_ENABLE =  0b0000_0010;
        const BG_ENABLE =      0b0000_0001;
    }
}

impl LCDControl {
    fn new() -> Self {
        Self::empty()
    }

    pub fn set_control(&mut self, val: u8) {
        self.remove(Self::all());
        self.insert(Self::from_bits_truncate(val));
    }
}

const HBLANK_MODE: u8 = 0;
const VBLANK_MODE: u8 = 1;
const OAM_MODE: u8 = 2;
const RENDER_MODE: u8 = 3;

pub struct LCDStatus {
    lyc_interrupt: bool,
    mode2_interrupt: bool,
    mode1_interrupt: bool,
    mode0_interrupt: bool,
    mode: u8,
}

impl LCDStatus {
    fn new() -> Self {
        Self {
            lyc_interrupt: false,
            mode2_interrupt: false,
            mode1_interrupt: false,
            mode0_interrupt: false,
            mode: 0,
        }
    }

    pub fn set_lyc_interrupt(&mut self, val: u8) {
      self.lyc_interrupt = val != 0
    }
    pub fn set_mode0_interrupt(&mut self, val: u8) {
      self.mode0_interrupt = val != 0
    }
    pub fn set_mode1_interrupt(&mut self, val: u8) {
      self.mode1_interrupt = val != 0
    }
    pub fn set_mode2_interrupt(&mut self, val: u8) {
      self.mode2_interrupt = val != 0
    }
    pub fn lyc_interrupt(&self) -> u8 {
      self.lyc_interrupt as u8
    }
    pub fn mode0_interrupt(&self) -> u8 {
      self.mode0_interrupt as u8
    }
    pub fn mode1_interrupt(&self) -> u8 {
      self.mode1_interrupt as u8
    }
    pub fn mode2_interrupt(&self) -> u8 {
      self.mode2_interrupt as u8
    }
    pub fn mode(&self) -> u8 {
      self.mode
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
}

// Currently, this just displays the tile data for the background tiles.
pub struct Ppu {
    display: Box<display::Display>,
    wait_for_frame: bool,
    // Video RAM. TODO(slongfield): In CGB, should be switchable banks.
    // Ox8000-0x9FFF
    vram: [u8; 0x2000],
    // Sprite attribute table.
    // 0xFE00-0xFE9F
    oam: [u8; 0x100],
    // I/O registers
    pub control: LCDControl,
    pub status: LCDStatus,
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
    // const OBP0: u16 = 0xFF48;
    // Object Palette 1 Data
    // const OBP1: u16 = 0xFF49;
    // Window Y and X position. This is an alternate background that is displayed above the
    // current background if visible.
    // const WY: u16 = 0xFF4A;
    // const WX: u16 = 0xFF4B;

    // Number of microseconds between frames.
    const INTERVAL: u64 = 16_666;

    pub fn new_sdl(video_subsystem: sdl2::VideoSubsystem) -> Self {
        Self {
            display: Box::new(sdl_display::SdlDisplay::new(video_subsystem)),
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
        if self.control.contains(LCDControl::ENABLE) {
            match self.status.mode {
                0 => self.mode0(interrupt),
                1 => self.mode1(interrupt),
                2 => self.mode2(interrupt),
                3 => self.mode3(),
                _ => unreachable!(),
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
                HBLANK_MODE | VBLANK_MODE | OAM_MODE => {
                    if let Some(old) = self.vram.get_mut((addr as usize) - 0x8000) {
                        *old = val;
                    }
                }
                RENDER_MODE => {}
                _ => unreachable!(),
            },
            addr @ 0xFE00..=0xFE9F => match self.status.mode {
                HBLANK_MODE | VBLANK_MODE => {
                    if let Some(old) = self.oam.get_mut((addr as usize) - 0xFE00) {
                        *old = val;
                    }
                }
                OAM_MODE | RENDER_MODE => {}
                _ => unreachable!(),
            },
            Self::LY => {}
            Self::LYC => self.lcd_y_compare = val,
            Self::DMA => {
                self.dma.enabled = true;
                self.dma.source = u16::from(val) * 0x100;
                self.dma.dest = 0xFE00;
            }
            Self::BGP => self.bg_palette = val,
            addr => info!("Attempted to write PPU with unmapped addr: {:#x}", addr),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            addr @ 0x8000..=0x9FFF => match self.status.mode {
                HBLANK_MODE | VBLANK_MODE | OAM_MODE => {
                    if let Some(val) = self.vram.get((addr as usize) - 0x8000) {
                        *val
                    } else {
                        error!("Read from unmapped VRAM address {:#04X}", addr);
                        0
                    }
                }
                RENDER_MODE => 0xFF,
                _ => unreachable!(),
            },
            addr @ 0xFE00..=0xFE9F => match self.status.mode {
                HBLANK_MODE | VBLANK_MODE => {
                    if let Some(val) = self.oam.get((addr as usize) - 0xFE00) {
                        *val
                    } else {
                        error!("Read from unmapped OAM address {:#04X}", addr);
                        0
                    }
                }
                OAM_MODE | RENDER_MODE => 0xFF,
                _ => unreachable!(),
            },
            Self::LY => self.lcd_y,
            Self::LYC => self.lcd_y_compare,
            Self::BGP => self.bg_palette,
            addr => {
                info!(
                    "Attempted to read from unhandled PPU register: {:#04X}",
                    addr
                );
                0
            }
        }
    }

    pub fn go_fast(&mut self) {
        self.wait_for_frame = false;
    }

    pub fn set_scroll_y(&mut self, val: u8) {
      self.scroll_y = val
    }


    pub fn set_scroll_x(&mut self, val: u8) {
      self.scroll_x = val
    }

    pub fn scroll_y(&self) -> u8 {
      self.scroll_y
    }

    pub fn scroll_x(&self) -> u8 {
      self.scroll_x
    }



    // HBlank, do nothing.
    fn mode0(&mut self, interrupt: &mut Interrupt) {
        self.mode_cycle += 1;
        if self.mode_cycle == MODE0_CYCLES {
            self.lcd_y += 1;
            self.update_ly_interrupt(interrupt);
            self.mode_cycle = 0;
            if self.lcd_y == VISIBLE_COUNT {
                self.status.mode = VBLANK_MODE;
            } else {
                self.status.mode = OAM_MODE;
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
                self.status.mode = OAM_MODE;
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
            self.status.mode = RENDER_MODE;
            self.update_mode_interrupt(interrupt);
        }
    }

    // Draw mode!
    fn mode3(&mut self) {
        // Only draw every other cycle, since we're drawing 8 pixels per cycle, but have 40 cycles
        // to draw 160 pixels.
        // TODO(slongfield): Model pixel fifo, or just draw a full line at a time.
        if self.mode_cycle % 2 == 0 {
            let mut pixels: [u8; 8] = [0; 8];
            let y = u16::from(self.scroll_y.wrapping_add(self.lcd_y));
            let x = u16::from(self.scroll_x.wrapping_add(self.mode_cycle * 4));
            let y_tile = y / 8;
            let x_tile = x / 8;
            // Get background pixels.
            {
                let tile_map_start: u16 = if self.control.contains(LCDControl::BG_TILE_MAP) {
                    0x1C00
                } else {
                    0x1800
                };
                let tile = self
                    .vram
                    .get((tile_map_start + y_tile * 32 + x_tile) as usize)
                    .unwrap_or(&0);
                let bg_tileset_start = if self.control.contains(LCDControl::BG_TILE_SET) {
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
            if self.control.contains(LCDControl::SPRITE_ENABLE) {
                // TODO(slongfield): Need to handle background pixel pallete data later, since
                // background 00 should draw over sprite pixels. Thankfully can ignore priority
                // for Tetris.
                let lcd_x = self.mode_cycle * 4;
                for sprite in &self.sprites {
                    // TODO(slongfield): Handle inverted sprites and double-tall sprites.
                    let tile_y = 7 - u16::from((sprite.y - self.lcd_y + 15) % 8);
                    if lcd_x + 8 <= sprite.x && lcd_x + 16 > sprite.x {
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
            self.status.mode = HBLANK_MODE;
        }
    }

    pub fn lcd_y_compare(&self) -> bool {
      self.lcd_y == self.lcd_y_compare
    }

    fn update_ly_interrupt(&mut self, interrupt: &mut Interrupt) {
        if self.status.lyc_interrupt && self.lcd_y_compare() {
            interrupt.set_lcd_stat_trigger(1)
        }
    }

    fn update_mode_interrupt(&mut self, interrupt: &mut Interrupt) {
        match self.status.mode {
            HBLANK_MODE => {
                if self.status.mode0_interrupt {
                    interrupt.set_lcd_stat_trigger(1)
                }
            }
            VBLANK_MODE => {
                if self.status.mode1_interrupt {
                    interrupt.set_lcd_stat_trigger(1)
                }
                interrupt.set_vblank_trigger(1)
            }
            OAM_MODE => {
                if self.status.mode2_interrupt {
                    interrupt.set_lcd_stat_trigger(1)
                }
            }
            RENDER_MODE => {}
            _ => unreachable!(),
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
