use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

mod cpu;

///! Wolfwig is the main object in the emulator that owns everything.
///! TODO(slongfield): Write some actual documentation.
pub struct Wolfwig {
    rom: Vec<u8>,
    header: cpu::header::Header,
}

impl Wolfwig {
    pub fn from_file(filename: &Path) -> Result<Wolfwig, io::Error> {
        let mut file = File::open(filename)?;
        let mut buffer = vec![];
        let read = file.read_to_end(&mut buffer)?;
        println!("Read {:x} bytes from {:?}", read, filename);

        let header = cpu::header::Header::new(&mut buffer);

        Ok(Wolfwig {
            rom: buffer,
            header: header,
        })
    }

    pub fn print_header(&self) {
        println!("{}", self.header);
    }
}