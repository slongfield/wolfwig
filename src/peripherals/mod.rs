use sdl2;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::sync::mpsc;

mod apu;
mod cartridge;
mod interrupt;
mod joypad;
pub mod mem;
mod ppu;
mod serial;
mod timer;

#[derive(Debug, Clone)]
pub struct Dma {
    pub enabled: bool,
    pub source: u16,
    pub dest: u16,
}

impl Dma {
    fn new() -> Self {
        Self {
            enabled: false,
            source: 0,
            dest: 0,
        }
    }
}

pub struct Peripherals {
    pub mem: mem::model::Memory,
    apu: apu::Apu,
    cartridge: Box<cartridge::Cartridge>,
    dma: Dma,
    interrupt: interrupt::Interrupt,
    joypad: joypad::Joypad,
    pub ppu: ppu::Ppu,
    serial: serial::Serial,
    timer: timer::Timer,
}

fn read_rom_from_file(filename: &Path) -> Result<Vec<u8>, io::Error> {
    let mut file = File::open(filename)?;
    let mut buffer = vec![];
    let read = file.read_to_end(&mut buffer)?;
    info!("Read {} bytes from {:?}", read, filename);
    Ok(buffer)
}

// Macro for fanning writes from a register out to various setters.
macro_rules! write_reg {
    ($val:ident: $( $msb:literal .. $lsb:literal =>
                    $self:ident.$mod:ident$(.$field:ident)+),* ) => {{
        $(
            $self.$mod$(.$field)+(($val & ((1 << ($msb-$lsb+1)) - 1 << $lsb)) >> $lsb);
        )*
    }}
}

// Macro for fanning reads from a reigster in from various getters. Unmapped bits are read as 1.
macro_rules! read_reg {
    ( $( $msb:literal .. $lsb:literal => $self:ident.$mod:ident$(.$field:ident)+),* ) => {{
        let mut val = 0xFF;
        $(
            val &= !(((1 << ($msb-$lsb+1)) - 1) << $lsb);
            val |= (u8::from($self.$mod$(.$field)+()) & ((1 << ($msb-$lsb+1)) - 1)) << $lsb;
        )*
            val
    }}
}

impl Peripherals {
    pub fn from_files(bootrom: &Path, rom: &Path) -> Result<Self, io::Error> {
        let bootrom = read_rom_from_file(bootrom)?;
        let rom = read_rom_from_file(rom)?;
        let sdl = sdl2::init().unwrap();
        let video_subsystem = sdl.video().unwrap();
        let ppu = ppu::Ppu::new_sdl(video_subsystem);
        let events = sdl.event_pump().unwrap();
        let joypad = joypad::Joypad::new_sdl(events);
        let audio_subsystem = sdl.audio().unwrap();
        let apu = apu::Apu::new(audio_subsystem);
        let interrupt = interrupt::Interrupt::new();
        let timer = timer::Timer::new();
        let dma = Dma::new();
        let cartridge = cartridge::new(bootrom, rom);
        Ok(Self {
            apu,
            cartridge,
            dma,
            interrupt,
            joypad,
            mem: mem::model::Memory::new(),
            ppu,
            serial: serial::Serial::new(None),
            timer,
        })
    }

    ///! Fake for testing.
    pub fn new_fake() -> Self {
        let ppu = ppu::Ppu::new_fake();
        let joypad = joypad::Joypad::new_fake();
        let apu = apu::Apu::new_fake();
        let interrupt = interrupt::Interrupt::new();
        let timer = timer::Timer::new();
        let dma = Dma::new();
        let cartridge = cartridge::new(vec![0; 0x100], vec![0; 0x1000]);
        Self {
            mem: mem::model::Memory::new(),
            serial: serial::Serial::new(None),
            cartridge,
            apu,
            ppu,
            joypad,
            interrupt,
            timer,
            dma,
        }
    }

    pub fn step(&mut self) {
        self.apu.step();
        self.joypad.step(&mut self.interrupt);
        self.ppu.step(&mut self.interrupt, &mut self.dma);
        self.serial.step();
        self.timer.step(&mut self.interrupt);
        if self.dma.enabled {
            // Disable dma for read
            self.dma.enabled = false;
            for index in 0..4 {
                let data = self.read(self.dma.source + index);
                let addr = self.dma.dest + index;
                self.write(addr, data);
            }
            self.dma.enabled = true;
        }
    }

    pub fn write(&mut self, address: u16, val: u8) {
        if self.dma.enabled {
            if let addr @ 0xFF80..=0xFFFE = address {
                self.mem.write(addr, val);
            }
        } else {
            match address {
                addr @ 0x0000..=0x7FFF | addr @ 0xFF50 => self.cartridge.write(addr, val),
                addr @ 0x8000..=0x9FFF | addr @ 0xFE00..=0xFE9F => self.ppu.write(addr, val),
                0xFF40 => self.ppu.control.set_control(val),
                0xFF41 => write_reg!(val:
                                     6..6 => self.ppu.status.set_lyc_interrupt,
                                     5..5 => self.ppu.status.set_mode2_interrupt,
                                     4..4 => self.ppu.status.set_mode1_interrupt,
                                     3..3 => self.ppu.status.set_mode0_interrupt
                ),
                0xFF42 => self.ppu.set_scroll_y(val),
                0xFF43 => self.ppu.set_scroll_x(val),
                0xFF44 => self.ppu.set_lcd_y(val),
                0xFF45 => self.ppu.set_lcd_y_compare(val),
                0xFF46 => self.ppu.set_dma(val),
                0xFF47 => write_reg!(val:
                                     7..6 => self.ppu.bg_palette.set_color3,
                                     5..4 => self.ppu.bg_palette.set_color2,
                                     3..2 => self.ppu.bg_palette.set_color1,
                                     1..0 => self.ppu.bg_palette.set_color0
                ),
                0xFF48 => write_reg!(val:
                                     7..6 => self.ppu.obj0_palette.set_color3,
                                     5..4 => self.ppu.obj0_palette.set_color2,
                                     3..2 => self.ppu.obj0_palette.set_color1,
                                     1..0 => self.ppu.obj0_palette.set_color0
                ),
                0xFF49 => write_reg!(val:
                                     7..6 => self.ppu.obj1_palette.set_color3,
                                     5..4 => self.ppu.obj1_palette.set_color2,
                                     3..2 => self.ppu.obj1_palette.set_color1,
                                     1..0 => self.ppu.obj1_palette.set_color0
                ),
                0xFF4A => self.ppu.set_window_y(val),
                0xFF4B => self.ppu.set_window_x(val),
                addr @ 0xA000..=0xBFFF
                | addr @ 0xC000..=0xCFFF
                | addr @ 0xD000..=0xDFFF
                | addr @ 0xFF80..=0xFFFE => self.mem.write(addr, val),
                // Echo RAM, maps back onto 0xC000-0XDDFF
                addr @ 0xE000..=0xFDFF => self.write(addr - 0x2000, val),
                addr @ 0xFEA0..=0xFEFF => info!("Write to unmapped memory region: {:#04X}", addr),
                // I/O registers.
                0xFF00 => {
                    write_reg!(val:
                               5..5 => self.joypad.set_select_button,
                               4..4 => self.joypad.set_select_direction
                    );
                    self.joypad.update(&mut self.interrupt);
                }
                0xFF01 => self.serial.set_data(val),
                0xFF02 => self.serial.set_start((1 << 7) & val != 0),
                0xFF04 => self.timer.set_divider(),
                0xFF05 => self.timer.set_counter(val),
                0xFF06 => self.timer.set_modulo(val),
                0xFF07 => write_reg!(val:
                                     2..2 => self.timer.set_start,
                                     1..0 => self.timer.set_input_clock
                ),
                0xFF0F => write_reg!(val:
                                     4..4 => self.interrupt.set_joypad_trigger,
                                     3..3 => self.interrupt.set_serial_trigger,
                                     2..2 => self.interrupt.set_timer_trigger,
                                     1..1 => self.interrupt.set_lcd_stat_trigger,
                                     0..0 => self.interrupt.set_vblank_trigger
                ),
                0xFF10 => write_reg!(val:
                                     6..4 => self.apu.channel_one.sweep.set_time,
                                     3..3 => self.apu.channel_one.sweep.set_direction,
                                     2..0 => self.apu.channel_one.sweep.set_shift
                ),
                0xFF11 => write_reg!(val:
                                     7..6 => self.apu.channel_one.length_pattern.set_duty,
                                     5..0 => self.apu.channel_one.length_pattern.set_length
                ),
                0xFF12 => write_reg!(val:
                                     7..4 => self.apu.channel_one.envelope.set_initial_volume,
                                     3..3 => self.apu.channel_one.envelope.set_direction,
                                     2..0 => self.apu.channel_one.envelope.set_sweep
                ),
                0xFF13 => self.apu.channel_one.frequency.set_frequency_low(val),
                0xFF14 => write_reg!(val:
                                     7..7 => self.apu.channel_one.frequency.set_start,
                                     6..6 => self.apu.channel_one.frequency.set_use_counter,
                                     2..0 => self.apu.channel_one.frequency.set_frequency_high
                ),
                0xFF16 => write_reg!(val:
                                     7..6 => self.apu.channel_two.length_pattern.set_duty,
                                     5..0 => self.apu.channel_two.length_pattern.set_length
                ),
                0xFF17 => write_reg!(val:
                                     7..4 => self.apu.channel_two.envelope.set_initial_volume,
                                     3..3 => self.apu.channel_two.envelope.set_direction,
                                     2..0 => self.apu.channel_two.envelope.set_sweep
                ),
                0xFF18 => self.apu.channel_two.frequency.set_frequency_low(val),
                0xFF19 => write_reg!(val:
                                     7..7 => self.apu.channel_two.frequency.set_start,
                                     6..6 => self.apu.channel_two.frequency.set_use_counter,
                                     2..0 => self.apu.channel_two.frequency.set_frequency_high
                ),
                0xFF1A => write_reg!(val:
                                     7..7 => self.apu.channel_three.set_enable
                ),
                0xFF1B => self.apu.channel_three.set_length(val),
                0xFF1C => write_reg!(val:
                                     6..5 => self.apu.channel_three.set_level
                ),
                0xFF1D => self.apu.channel_three.frequency.set_frequency_low(val),
                0xFF1E => write_reg!(val:
                                     7..7 => self.apu.channel_three.frequency.set_start,
                                     6..6 => self.apu.channel_three.frequency.set_use_counter,
                                     2..0 => self.apu.channel_three.frequency.set_frequency_high
                ),
                addr @ 0xFF30..=0xFF3F => self
                    .apu
                    .channel_three
                    .set_table(usize::from(0xFF30 - addr), val),
                0xFF20 => write_reg!(val:
                                     5..0 => self.apu.channel_four.set_length
                ),
                0xFF21 => write_reg!(val:
                                     7..4 => self.apu.channel_four.envelope.set_initial_volume,
                                     3..3 => self.apu.channel_four.envelope.set_direction,
                                     2..0 => self.apu.channel_four.envelope.set_sweep
                ),
                0xFF22 => write_reg!(val:
                                     7..4 => self.apu.channel_four.counter.set_frequency,
                                     3..3 => self.apu.channel_four.counter.set_width,
                                     2..0 => self.apu.channel_four.counter.set_ratio
                ),
                0xFF23 => write_reg!(val:
                                     7..7 => self.apu.channel_four.set_start,
                                     6..6 => self.apu.channel_four.set_stop_on_length
                ),
                0xFF24 => write_reg!(val:
                                     6..4 => self.apu.control.volume.set_left,
                                     2..0 => self.apu.control.volume.set_right
                ),
                0xFF25 => self.apu.control.channel_enable.set_enable(val),
                0xFF26 => write_reg!(val:
                                     7..7 => self.apu.control.set_enable
                ),
                0xFF03 | 0xFF08..=0xFF0E | 0xFF4C..=0xFF4F | 0xFF50..=0xFF7F => {
                    info!("Write to unmapped I/O reg!")
                }
                0xFFFF => write_reg!(val:
                                     7..5 => self.interrupt.set_unused,
                                     4..4 => self.interrupt.set_joypad_enable,
                                     3..3 => self.interrupt.set_serial_enable,
                                     2..2 => self.interrupt.set_timer_enable,
                                     1..1 => self.interrupt.set_lcd_stat_enable,
                                     0..0 => self.interrupt.set_vblank_enable
                ),
                _ => {}
            }
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        if self.dma.enabled {
            match address {
                addr @ 0xFF80..=0xFFFE => self.mem.read(addr),
                _ => 0xFF,
            }
        } else {
            match address {
                addr @ 0x0000..=0x7FFF | addr @ 0xFF50 => self.cartridge.read(addr),
                addr @ 0x8000..=0x9FFF | addr @ 0xFE00..=0xFE9F => self.ppu.read(addr),
                0xFF40 => self.ppu.control.bits(),
                0xFF41 => read_reg!(
                    6..6 => self.ppu.status.lyc_interrupt,
                    5..5 => self.ppu.status.mode2_interrupt,
                    4..4 => self.ppu.status.mode1_interrupt,
                    3..3 => self.ppu.status.mode0_interrupt,
                    2..2 => self.ppu.lcd_y_compare,
                    1..0 => self.ppu.status.mode
                ),
                0xFF42 => self.ppu.scroll_y(),
                0xFF43 => self.ppu.scroll_x(),
                0xFF44 => self.ppu.lcd_y(),
                0xFF45 => self.ppu.lcd_y_compare(),
                0xFF46 => 0xFF, // TODO(slongfield): What does DMA read do?
                0xFF47 => read_reg!(
                    7..6 => self.ppu.bg_palette.color3,
                    5..4 => self.ppu.bg_palette.color2,
                    3..2 => self.ppu.bg_palette.color1,
                    1..0 => self.ppu.bg_palette.color0
                ),
                0xFF48 => read_reg!(
                    7..6 => self.ppu.obj0_palette.color3,
                    5..4 => self.ppu.obj0_palette.color2,
                    3..2 => self.ppu.obj0_palette.color1,
                    1..0 => self.ppu.obj0_palette.color0
                ),
                0xFF49 => read_reg!(
                    7..6 => self.ppu.obj1_palette.color3,
                    5..4 => self.ppu.obj1_palette.color2,
                    3..2 => self.ppu.obj1_palette.color1,
                    1..0 => self.ppu.obj1_palette.color0
                ),
                0xFF4A => self.ppu.window_y(),
                0xFF4B => self.ppu.window_x(),
                addr @ 0xA000..=0xBFFF
                | addr @ 0xC000..=0xCFFF
                | addr @ 0xD000..=0xDFFF
                | addr @ 0xFF80..=0xFFFE => self.mem.read(addr),
                // Echo RAM, maps back onto 0xC000-0XDDFF
                addr @ 0xE000..=0xFDFF => self.read(addr - 0x2000),
                addr @ 0xFEA0..=0xFEFF => {
                    info!("Read from unmapped memory region: {:#04X}", addr);
                    0
                }
                0xFF00 => read_reg!(
                    5..5 => self.joypad.select_direction,
                    4..4 => self.joypad.select_button,
                    3..0 => self.joypad.state
                ),
                0xFF01 => self.serial.data(),
                0xFF02 => read_reg!(7..7 => self.serial.start),
                0xFF04 => self.timer.divider(),
                0xFF05 => self.timer.counter(),
                0xFF06 => self.timer.modulo(),
                0xFF07 => read_reg!(
                    2..2 => self.timer.start,
                    1..0 => self.timer.input_clock
                ),
                0xFF0F => read_reg!(
                    4..4 => self.interrupt.joypad_trigger,
                    3..3 => self.interrupt.serial_trigger,
                    2..2 => self.interrupt.timer_trigger,
                    1..1 => self.interrupt.lcd_stat_trigger,
                    0..0 => self.interrupt.vblank_trigger
                ),
                0xFF10 => read_reg!(
                    6..4 => self.apu.channel_one.sweep.time,
                    3..3 => self.apu.channel_one.sweep.direction,
                    2..0 => self.apu.channel_one.sweep.shift
                ),
                0xFF11 => read_reg!(
                    7..6 => self.apu.channel_one.length_pattern.duty,
                    5..0 => self.apu.channel_one.length_pattern.length
                ),
                0xFF12 => read_reg!(
                    7..4 => self.apu.channel_one.envelope.initial_volume,
                    3..3 => self.apu.channel_one.envelope.direction,
                    2..0 => self.apu.channel_one.envelope.sweep
                ),
                0xFF13 => self.apu.channel_one.frequency.frequency_low(),
                0xFF14 => read_reg!(
                    7..7 => self.apu.channel_one.frequency.start,
                    6..6 => self.apu.channel_one.frequency.use_counter,
                    2..0 => self.apu.channel_one.frequency.frequency_high
                ),
                0xFF16 => read_reg!(
                    7..6 => self.apu.channel_two.length_pattern.duty,
                    5..0 => self.apu.channel_two.length_pattern.length
                ),
                0xFF17 => read_reg!(
                    7..4 => self.apu.channel_two.envelope.initial_volume,
                    3..3 => self.apu.channel_two.envelope.direction,
                    2..0 => self.apu.channel_two.envelope.sweep
                ),
                0xFF18 => self.apu.channel_two.frequency.frequency_low(),
                0xFF19 => read_reg!(
                    7..7 => self.apu.channel_two.frequency.start,
                    6..6 => self.apu.channel_two.frequency.use_counter,
                    2..0 => self.apu.channel_two.frequency.frequency_high
                ),
                0xFF1A => read_reg!(
                    7..7 => self.apu.channel_three.enable
                ),
                0xFF1B => self.apu.channel_three.length(),
                0xFF1C => read_reg!(
                    6..5 => self.apu.channel_three.level
                ),
                0xFF1D => self.apu.channel_three.frequency.frequency_low(),
                0xFF1E => read_reg!(
                    7..7 => self.apu.channel_three.frequency.start,
                    6..6 => self.apu.channel_three.frequency.use_counter,
                    2..0 => self.apu.channel_three.frequency.frequency_high
                ),
                addr @ 0xFF30..=0xFF3F => self.apu.channel_three.table(usize::from(0xFF30 - addr)),
                0xFF20 => read_reg!(
                    5..0 => self.apu.channel_four.length
                ),
                0xFF21 => read_reg!(
                    7..4 => self.apu.channel_four.envelope.initial_volume,
                    3..3 => self.apu.channel_four.envelope.direction,
                    2..0 => self.apu.channel_four.envelope.sweep
                ),
                0xFF22 => read_reg!(
                    7..4 => self.apu.channel_four.counter.frequency,
                    3..3 => self.apu.channel_four.counter.width,
                    2..0 => self.apu.channel_four.counter.ratio
                ),
                0xFF23 => read_reg!(
                    6..6 => self.apu.channel_four.stop_on_length
                ),
                0xFF24 => read_reg!(
                    6..4 => self.apu.control.volume.left,
                    2..0 => self.apu.control.volume.right
                ),
                0xFF25 => self.apu.control.channel_enable.enable(),
                0xFF26 => read_reg!(
                    7..7 => self.apu.control.enable,
                    3..3 => self.apu.channel_four.active,
                    2..2 => self.apu.channel_three.active,
                    1..1 => self.apu.channel_two.active,
                    0..0 => self.apu.channel_one.active
                ),
                0xFF03
                | 0xFF08..=0xFF0E
                | 0xFF15
                | 0xFF1F
                | 0xFF27..=0xFF2F
                | 0xFF4C..=0xFF4F
                | 0xFF50..=0xFF7F => {
                    info!("Read from unmapped I/O reg!");
                    0xFF
                }
                0xFFFF => read_reg!(
                    7..5 => self.interrupt.unused,
                    4..4 => self.interrupt.joypad_enable,
                    3..3 => self.interrupt.serial_enable,
                    2..2 => self.interrupt.timer_enable,
                    1..1 => self.interrupt.lcd_stat_enable,
                    0..0 => self.interrupt.vblank_enable
                ),
            }
        }
    }

    pub fn get_interrupt(&self) -> Option<u16> {
        self.interrupt.get_interrupt_pc()
    }

    pub fn disable_interrupt(&mut self) {
        self.interrupt.disable_interrupt()
    }

    pub fn connect_serial_channel(&mut self, tx: mpsc::Sender<u8>) {
        self.serial.connect_channel(tx);
    }

    pub fn print_header(&self) {
        println!("{}", self.cartridge);
    }

    pub fn go_fast(&mut self) {
        self.ppu.go_fast();
    }
}
