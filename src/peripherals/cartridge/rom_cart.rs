///!Pure ROM cartridge.
use peripherals::cartridge::header;
use peripherals::cartridge::Cartridge;
use std::fmt;

pub struct RomCart {
    bootrom: Vec<u8>,
    rom: Vec<u8>,
    bootrom_disabled: bool,
}

impl RomCart {
    pub fn new(bootrom: Vec<u8>, rom: Vec<u8>) -> Self {
        Self {
            bootrom,
            rom,
            bootrom_disabled: false,
        }
    }
}

impl Cartridge for RomCart {
    fn read(&self, address: u16) -> u8 {
        match address {
            addr @ 0x000..=0x100 if !self.bootrom_disabled => {
                *self.bootrom.get(addr as usize).unwrap_or(&0xFF)
            }
            0xFF50 => self.bootrom_disabled as u8,
            addr => *self.rom.get(addr as usize).unwrap_or(&0xFF),
        }
    }

    fn write(&mut self, address: u16, val: u8) {
        if address == 0xFF50 {
            self.bootrom_disabled = val != 0;
        }
    }
}

impl fmt::Display for RomCart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let header = header::Header::new(&self.rom);
        write!(f, "{}", header)
    }
}
