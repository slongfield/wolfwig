pub mod header;

mod mbc_one;
mod rom_cart;

use std::fmt;

pub fn new(bootrom: Vec<u8>, rom: Vec<u8>) -> Box<Cartridge> {
    let header = header::Header::new(&rom);
    match header.cartridge_type {
        header::CartridgeType::ROM => Box::new(rom_cart::RomCart::new(bootrom, rom)),
        header::CartridgeType::MBC1 => Box::new(mbc_one::MbcOne::new(bootrom, rom)),
        other => panic!("Unhandled cartridge type: {:?}", other),
    }
}

pub trait Cartridge: fmt::Display {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, val: u8);
}
