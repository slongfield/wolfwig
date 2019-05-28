#[macro_use]
extern crate log;
#[macro_use]
extern crate bitflags;

extern crate sdl2;

use std::io::{self, stdout, Write};
use std::path::Path;
use std::sync::mpsc;
use std::thread;

pub mod debug;

mod cpu;
mod peripherals;
mod util;

///! Wolfwig is the main object in the emulator that owns everything.
///! TODO(slongfield): Write some actual documentation.
pub struct Wolfwig {
    pub peripherals: peripherals::Peripherals,
    cpu: cpu::lr25902::LR25902,
}

impl Wolfwig {
    pub fn from_files(bootrom: &Path, rom: &Path) -> Result<Self, io::Error> {
        let peripherals = peripherals::Peripherals::from_files(bootrom, rom)?;

        Ok(Self {
            peripherals,
            cpu: cpu::lr25902::LR25902::new(),
        })
    }

    pub fn step(&mut self) -> bool {
        self.peripherals.step();
        self.cpu.step(&mut self.peripherals)
    }

    pub fn start_print_serial(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.peripherals.connect_serial_channel(tx);
        thread::spawn(move || loop {
            let received = rx.recv().unwrap();
            print!("{}", char::from(received));
            stdout().flush().expect("Could not flush stdout");
        });
    }

    pub fn print_header(&self) {
        self.peripherals.print_header();
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

    pub fn go_fast(&mut self) {
        self.peripherals.go_fast();
    }
}
