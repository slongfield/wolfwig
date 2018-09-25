#[macro_use]
extern crate log;

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

mod cpu;
mod mem;
mod serial;
mod util;

///! Wolfwig is the main object in the emulator that owns everything.
///! TODO(slongfield): Write some actual documentation.
pub struct Wolfwig {
    cpu: cpu::lr25902::LR25902,
    mem: mem::model::Memory,
    serial: serial::Serial,
}

fn read_rom_from_file(filename: &Path) -> Result<Vec<u8>, io::Error> {
    let mut file = File::open(filename)?;
    let mut buffer = vec![];
    let read = file.read_to_end(&mut buffer)?;
    info!("Read {} bytes from {:?}", read, filename);
    Ok(buffer)
}

impl Wolfwig {
    pub fn from_files(bootrom: &Path, rom: &Path) -> Result<Wolfwig, io::Error> {
        let bootrom = read_rom_from_file(bootrom)?;
        let rom = read_rom_from_file(rom)?;
        Ok(Wolfwig {
            mem: mem::model::Memory::new(bootrom, rom),
            cpu: cpu::lr25902::LR25902::new(),
            serial: serial::Serial::new(None),
        })
    }

    pub fn step(&mut self) -> u16 {
        self.serial.step(&mut self.mem);
        self.cpu.step(&mut self.mem)
    }

    pub fn print_header(&self) {
        println!("{}", self.mem.header);
    }

    pub fn dump_instructions(&self, start_pc: usize, end_pc: usize) {
        self.cpu.dump_instructions(&self.mem, start_pc, end_pc);
    }
}
