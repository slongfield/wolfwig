#[macro_use]
extern crate log;

extern crate sdl2;

use std::fs::File;
use std::io::{self, stdout, Read, Write};
use std::path::Path;
use std::sync::mpsc;
use std::thread;

pub mod debug;

mod cpu;
mod joypad;
mod mem;
mod ppu;
mod serial;
mod timer;
mod util;

///! Wolfwig is the main object in the emulator that owns everything.
///! TODO(slongfield): Write some actual documentation.
pub struct Wolfwig {
    pub mem: mem::model::Memory,
    sdl: sdl2::Sdl,
    cpu: cpu::lr25902::LR25902,
    serial: serial::Serial,
    ppu: ppu::Ppu,
    joypad: joypad::Joypad,
}

fn read_rom_from_file(filename: &Path) -> Result<Vec<u8>, io::Error> {
    let mut file = File::open(filename)?;
    let mut buffer = vec![];
    let read = file.read_to_end(&mut buffer)?;
    info!("Read {} bytes from {:?}", read, filename);
    Ok(buffer)
}

impl Wolfwig {
    pub fn from_files(bootrom: &Path, rom: &Path) -> Result<Self, io::Error> {
        let bootrom = read_rom_from_file(bootrom)?;
        let rom = read_rom_from_file(rom)?;
        let sdl = sdl2::init().unwrap();
        let video_subsystem = sdl.video().unwrap();
        let event_subsystem = sdl.event_pump().unwrap();

        let ppu = ppu::Ppu::new(video_subsystem);
        let joypad = joypad::Joypad::new(event_subsystem);
        Ok(Self {
            sdl,
            mem: mem::model::Memory::new(bootrom, rom),
            cpu: cpu::lr25902::LR25902::new(),
            serial: serial::Serial::new(None),
            ppu,
            joypad,
        })
    }

    pub fn step(&mut self) -> bool {
        self.serial.step(&mut self.mem);
        self.ppu.step(&mut self.mem);
        self.joypad.step(&mut self.mem);
        self.cpu.step(&mut self.mem)
    }

    pub fn start_print_serial(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.serial.connect_channel(tx);
        thread::spawn(move || loop {
            let received = rx.recv().unwrap();
            //println!("Serial: 0x{:02X} ({})", received, char::from(received));
            print!("{}", char::from(received));
            stdout().flush().expect("Could not flush stdout");
        });
    }

    pub fn print_header(&self) {
        println!("{}", self.mem.header);
    }

    pub fn print_registers(&self) {
        println!("{}", self.cpu.regs);
    }

    pub fn pc(&self) -> u16 {
        self.cpu.pc()
    }

    pub fn print_reg8(&self, reg: cpu::registers::Reg8) {
        println!("0x{:02X}", self.cpu.regs.read8(reg));
    }

    pub fn print_reg16(&self, reg: cpu::registers::Reg16) {
        println!("0x{:02X}", self.cpu.regs.read16(reg));
    }

    pub fn dump_instructions(&self, start_pc: usize, end_pc: usize) {
        self.cpu.dump_instructions(&self.mem, start_pc, end_pc);
    }
}
