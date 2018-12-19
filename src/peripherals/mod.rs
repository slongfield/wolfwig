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

#[derive(Debug)]
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
    ppu: ppu::Ppu,
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

impl Peripherals {
    pub fn from_files(bootrom: &Path, rom: &Path) -> Result<Self, io::Error> {
        let bootrom = read_rom_from_file(bootrom)?;
        let rom = read_rom_from_file(rom)?;
        let sdl = sdl2::init().unwrap();
        let video_subsystem = sdl.video().unwrap();
        let ppu = ppu::Ppu::new_sdl(video_subsystem);
        let events = sdl.event_pump().unwrap();
        let joypad = joypad::Joypad::new_sdl(events);
        let apu = apu::Apu::new();
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
        let apu = apu::Apu::new();
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
        self.joypad.step();
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
                addr @ 0x0000..=0x7FFF => self.cartridge.write(addr, val),
                addr @ 0x8000..=0x9FFF => self.ppu.write(addr, val),
                addr @ 0xA000..=0xBFFF => self.mem.write(addr, val),
                addr @ 0xC000..=0xCFFF => self.mem.write(addr, val),
                addr @ 0xD000..=0xDFFF => self.mem.write(addr, val),
                // Echo RAM, maps back onto 0xC000-0XDDFF
                addr @ 0xE000..=0xFDFF => self.write(addr - 0x2000, val),
                addr @ 0xFE00..=0xFE9F => self.ppu.write(addr, val),
                addr @ 0xFEA0..=0xFEFF => info!("Write to unmapped memory region: {:#04X}", addr),
                // I/O registers.
                addr @ 0xFF00 => self.joypad.write(addr, val),
                addr @ 0xFF01..=0xFF02 => self.serial.write(addr, val),
                addr @ 0xFF04..=0xFF07 => self.timer.write(addr, val),
                addr @ 0xFF0F => self.interrupt.write(addr, val),
                addr @ 0xFF10..=0xFF3F => self.apu.write(addr, val),
                addr @ 0xFF40..=0xFF4B => self.ppu.write(addr, val),
                addr @ 0xFF50 => self.cartridge.write(addr, val),
                0xFF03 | 0xFF08..=0xFF0E | 0xFF4C..=0xFF4F | 0xFF50..=0xFF79 => {
                    info!("Write to unmapped I/O reg!")
                }
                // High RAM.
                addr @ 0xFF80..=0xFFFE => self.mem.write(addr, val),
                addr @ 0xFFFF => self.interrupt.write(addr, val),
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
                addr @ 0x0000..=0x7FFF => self.cartridge.read(addr),
                addr @ 0x8000..=0x9FFF => self.ppu.read(addr),
                addr @ 0xA000..=0xBFFF => self.mem.read(addr),
                addr @ 0xC000..=0xCFFF => self.mem.read(addr),
                addr @ 0xD000..=0xDFFF => self.mem.read(addr),
                addr @ 0xE000..=0xFDFF => self.read(addr - 0x2000),
                addr @ 0xFE00..=0xFE9F => self.ppu.read(addr),
                addr @ 0xFEA0..=0xFEFF => {
                    info!("Read from unmapped memory region: {:#04X}", addr);
                    0
                }
                addr @ 0xFF00 => self.joypad.read(addr),
                addr @ 0xFF01..=0xFF02 => self.serial.read(addr),
                addr @ 0xFF04..=0xFF07 => self.timer.read(addr),
                addr @ 0xFF0F => self.interrupt.read(addr),
                addr @ 0xFF10..=0xFF3F => self.apu.read(addr),
                addr @ 0xFF40..=0xFF4B => self.ppu.read(addr),
                addr @ 0xFF50 => self.cartridge.read(addr),
                0xFF03 | 0xFF08..=0xFF0E | 0xFF4C..=0xFF4F | 0xFF50..=0xFF79 => {
                    info!("Read from unmapped I/O reg!");
                    0
                }
                addr @ 0xFF80..=0xFFFE => self.mem.read(addr),
                addr @ 0xFFFF => self.interrupt.read(addr),
                _ => 0,
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
}
