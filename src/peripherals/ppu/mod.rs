use peripherals::interrupt::Interrupt;
use peripherals::Dma;
use sdl2;
use std::thread;
use std::time::{Duration, Instant};

mod display;
mod fake_display;
mod sdl_display;

const LINE_COUNT: u8 = 154;
const VISIBLE_COUNT: u8 = 144;
const PIXEL_WIDTH: usize = 160;
const MODE0_CYCLES: u8 = 51;
const MODE1_CYCLES: u8 = 114; // cycles per line
const MODE2_CYCLES: u8 = 20;
const MODE3_CYCLES: u8 = 43;

bitflags! {
    pub struct LCDControl: u8 {
        const ENABLE =          0b1000_0000;
        const WINDOW_TILE_MAP = 0b0100_0000;
        const WINDOW_DISPLAY =  0b0010_0000;
        const BG_TILE_SET =     0b0001_0000;
        const BG_TILE_MAP =     0b0000_1000;
        const SPRITE_SIZE =     0b0000_0100;
        const SPRITE_ENABLE =   0b0000_0010;
        const BG_ENABLE =       0b0000_0001;
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

    fn bg_tile_map(&self) -> usize {
        if self.contains(LCDControl::BG_TILE_MAP) {
            0x1C00
        } else {
            0x1800
        }
    }

    fn bg_tile_addr(&self, tile_number: u8) -> usize {
        if self.contains(LCDControl::BG_TILE_SET) || tile_number > 127 {
            usize::from(tile_number) * 16
        } else {
            usize::from(0x1000 + usize::from(tile_number) * 16)
        }
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

pub struct Palette {
    color0: u8,
    color1: u8,
    color2: u8,
    color3: u8,
}

impl Palette {
    fn new() -> Self {
        Self {
            color0: 0,
            color1: 0,
            color2: 0,
            color3: 0,
        }
    }
    pub fn set_color0(&mut self, val: u8) {
        self.color0 = val;
    }
    pub fn set_color1(&mut self, val: u8) {
        self.color1 = val;
    }

    pub fn set_color2(&mut self, val: u8) {
        self.color2 = val;
    }

    pub fn set_color3(&mut self, val: u8) {
        self.color3 = val;
    }

    pub fn color0(&self) -> u8 {
        self.color0
    }
    pub fn color1(&self) -> u8 {
        self.color1
    }
    pub fn color2(&self) -> u8 {
        self.color2
    }
    pub fn color3(&self) -> u8 {
        self.color3
    }

    fn get_color(&self, key: u8) -> u8 {
        match key {
            0 => self.color0,
            1 => self.color1,
            2 => self.color2,
            3 => self.color3,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
struct Tile {
    data: Vec<u8>,
}

impl Tile {
    fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    fn pixel(&self, x: usize, y: usize) -> u8 {
        let x = 7 - x;
        let high_bit = (self.data[y * 2 + 1] & (1 << x)) >> x;
        let low_bit = (self.data[y * 2] & (1 << x)) >> x;
        (high_bit << 1) | low_bit
    }
}

#[derive(Debug)]
struct Sprite {
    pub tile: Tile,
    x: usize,
    y: usize,
    flags: u8,
}

impl Sprite {
    fn new(tile: Tile, x: u8, y: u8, flags: u8) -> Self {
        Self {
            tile: tile,
            x: usize::from(x),
            y: usize::from(y),
            flags: flags,
        }
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
    window_x: u8,
    window_y: u8,
    lcd_y: u8,
    lcd_y_compare: u8,
    pub bg_palette: Palette,
    pub obj0_palette: Palette,
    pub obj1_palette: Palette,
    mode_cycle: u8,
    sprites: Vec<Sprite>,
    before: Instant,
    dma: Dma,
}

impl Ppu {
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
            window_x: 0,
            window_y: 0,
            lcd_y_compare: 0,
            control: LCDControl::new(),
            status: LCDStatus::new(),
            bg_palette: Palette::new(),
            obj0_palette: Palette::new(),
            obj1_palette: Palette::new(),
            mode_cycle: 0,
            sprites: vec![],
            before: Instant::now(),
            dma: Dma::new(),
        }
    }

    pub fn new_fake() -> Self {
        Self {
            display: Box::new(fake_display::FakeDisplay::new()),
            wait_for_frame: true,
            vram: [0; 0x2000],
            oam: [0; 0x100],
            lcd_y: 0,
            scroll_x: 0,
            scroll_y: 0,
            window_x: 0,
            window_y: 0,
            lcd_y_compare: 0,
            control: LCDControl::new(),
            status: LCDStatus::new(),
            bg_palette: Palette::new(),
            obj0_palette: Palette::new(),
            obj1_palette: Palette::new(),
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
                3 => self.render_line(),
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
            *dma = self.dma.clone();
            self.dma = Dma::new();
        }
    }

    pub fn set_lcd_y(&mut self, val: u8) {
        self.lcd_y = val & 0
    }

    pub fn lcd_y(&self) -> u8 {
        self.lcd_y
    }

    pub fn set_dma(&mut self, val: u8) {
        self.dma.enabled = true;
        self.dma.source = u16::from(val) * 0x100;
        self.dma.dest = 0xFE00;
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

    pub fn set_window_y(&mut self, val: u8) {
        self.window_y = val
    }

    pub fn set_window_x(&mut self, val: u8) {
        self.window_x = val
    }

    pub fn window_y(&self) -> u8 {
        self.window_y
    }

    pub fn window_x(&self) -> u8 {
        self.window_x
    }

    // HBlank, don't render anything, go to VBLANK or OAM mode at end of cycle.
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

    // VBlank, don't render anything, go to OAM mode at end of cycles.
    fn mode1(&mut self, interrupt: &mut Interrupt) {
        self.mode_cycle += 1;
        if self.mode_cycle == MODE1_CYCLES {
            self.lcd_y += 1;
            self.update_ly_interrupt(interrupt);
            self.mode_cycle = 0;
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

    // OAM mode, build sprite list.
    fn mode2(&mut self, interrupt: &mut Interrupt) {
        if self.mode_cycle == 0 {
            self.sprites = vec![];
            for entry in self.oam.chunks(4) {
                let y = *entry.get(0).unwrap_or(&0);
                let x = *entry.get(1).unwrap_or(&0);
                let tile_number = *entry.get(2).unwrap_or(&0);
                // TODO(slongfield): Handle double-tall tiles.
                let tile = Tile::new(
                    (0..16)
                        .map(|offset| {
                            *self
                                .vram
                                .get(usize::from(tile_number) * 16 + offset)
                                .unwrap_or(&0)
                        })
                        .collect::<Vec<u8>>(),
                );
                let flags = *entry.get(3).unwrap_or(&0);
                // Only add the sprite if it'll be visibile.
                if self.lcd_y + 8 < y && self.lcd_y + 16 >= y {
                    self.sprites.push(Sprite::new(tile, x, y, flags));
                }
            }
            // Sort by X, since smallest X gets highest priority, so want to draw it
            // first.
            self.sprites.sort_unstable_by(|a, b| (a.x).cmp(&b.x));
        }
        self.mode_cycle += 1;
        if self.mode_cycle == MODE2_CYCLES {
            self.mode_cycle = 0;
            self.status.mode = RENDER_MODE;
            self.update_mode_interrupt(interrupt);
        }
    }

    // Render mode, draw a line.
    fn render_line(&mut self) {
        if self.mode_cycle != 0 {
            self.mode_cycle += 1;
            if self.mode_cycle == MODE3_CYCLES {
                self.mode_cycle = 0;
                self.status.mode = HBLANK_MODE;
            }
            return;
        }
        let mut pixels: [u8; PIXEL_WIDTH] = [0; PIXEL_WIDTH];
        // Set up the background.
        {
            let bg_y = usize::from(self.scroll_y.wrapping_add(self.lcd_y));
            let y_offset = (bg_y / 8) * 32;
            let tiles = (0..32)
                .map(|line_offset| {
                    *self
                        .vram
                        .get(self.control.bg_tile_map() + y_offset + line_offset)
                        .unwrap_or(&0)
                })
                .map(|tile_number| {
                    let base_addr = self.control.bg_tile_addr(tile_number);
                    Tile::new(
                        (0..16)
                            .map(|offset| *self.vram.get(base_addr + offset).unwrap_or(&0))
                            .collect::<Vec<u8>>(),
                    )
                })
                .collect::<Vec<Tile>>();
            for offset in 0..160 {
                let x = usize::from(self.scroll_x.wrapping_add(offset));
                let tile = tiles.get(x / 8).unwrap();
                pixels[usize::from(offset)] = tile.pixel(x % 8, bg_y % 8);
            }
        }
        // Set up the window.
        // Set up the sprites and select colors.
        {
            if !self.control.contains(LCDControl::SPRITE_ENABLE) || self.sprites.len() == 0 {
                for pixel in pixels.iter_mut() {
                    *pixel = self.bg_palette.get_color(*pixel);
                }
            } else {
                let mut sprite_offset = 0;
                for (index, pixel) in pixels.iter_mut().enumerate() {
                    if self.control.contains(LCDControl::SPRITE_ENABLE)
                        && sprite_offset <= self.sprites.len()
                        && sprite_offset < 9
                    {
                        if self.sprites.len() > (sprite_offset + 1)
                            && (index + 8) >= usize::from(self.sprites[sprite_offset + 1].x)
                        {
                            sprite_offset += 1;
                        }
                        let sprite = self.sprites.get(sprite_offset).unwrap();
                        if sprite.x > index && sprite.x <= index + 8 {
                            // TODO(slongfield): Handle double-tall sprites.
                            let tile_y = (usize::from(self.lcd_y) - sprite.y + 16) % 8;
                            let tile_x = (index - sprite.x) % 8;
                            // TODO(slongfield): Handle flags.
                            *pixel = self.bg_palette.get_color(sprite.tile.pixel(tile_x, tile_y));
                        } else {
                            *pixel = self.bg_palette.get_color(*pixel);
                        }
                    } else {
                        *pixel = self.bg_palette.get_color(*pixel);
                    }
                }
            }
        }
        // Draw the line.
        for (index, pixel) in pixels.iter().enumerate() {
            // TODO(slongfield): Adjust to taste.
            let color = match pixel {
                0b00 => display::Color::RGB(155, 188, 15),
                0b01 => display::Color::RGB(48, 98, 48),
                0b10 => display::Color::RGB(139, 172, 15),
                _ => display::Color::RGB(15, 56, 15),
            };
            self.display
                .draw_pixel(index as usize, self.lcd_y as usize, color)
                .expect("Could not draw rectangle");
        }
        self.mode_cycle += 1;
    }

    pub fn check_lcd_y_compare(&self) -> bool {
        self.lcd_y == self.lcd_y_compare
    }

    pub fn set_lcd_y_compare(&mut self, val: u8) {
        self.lcd_y_compare = val
    }

    pub fn lcd_y_compare(&self) -> u8 {
        self.lcd_y_compare
    }

    fn update_ly_interrupt(&mut self, interrupt: &mut Interrupt) {
        if self.status.lyc_interrupt && self.check_lcd_y_compare() {
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
}
