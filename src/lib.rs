#[macro_use]
extern crate log;

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

mod cpu;
mod mem;
mod util;

///! Wolfwig is the main object in the emulator that owns everything.
///! TODO(slongfield): Write some actual documentation.
pub struct Wolfwig {
    cpu: cpu::lr25902::LR25902,
    mem: mem::model::Memory,
}

impl Wolfwig {
    pub fn from_file(filename: &Path) -> Result<Wolfwig, io::Error> {
        let mut file = File::open(filename)?;
        let mut buffer = vec![];
        let read = file.read_to_end(&mut buffer)?;
        info!("Read {} bytes from {:?}", read, filename);

        Ok(Wolfwig {
            mem: mem::model::Memory::new(buffer),
            cpu: cpu::lr25902::LR25902::new(),
        })
    }

    pub fn print_header(&self) {
        println!("{}", self.mem.header);
    }

    pub fn dump_instructions(&self, start_pc: usize, end_pc: usize) {
        self.cpu.dump_instructions(&self.mem.rom, start_pc, end_pc);
    }
}
