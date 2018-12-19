///!Model of an MBC1 cartridge.
use peripherals::cartridge::header;
use peripherals::cartridge::Cartridge;
use std::fmt;

pub struct MbcOne {
    bootrom: Vec<u8>,
    rom: Vec<u8>,
    bootrom_disabled: bool,
    ram: Vec<u8>,
    rom_bank: u8,
    ram_bank: u8,
    rom_ram_mode: bool,
}

impl MbcOne {
    pub fn new(bootrom: Vec<u8>, rom: Vec<u8>) -> Self {
        Self {
            bootrom,
            rom,
            bootrom_disabled: false,
            ram: vec![0; 0x2000],
            rom_bank: 0,
            ram_bank: 0,
            rom_ram_mode: false,
        }
    }
}

impl Cartridge for MbcOne {
    fn read(&self, address: u16) -> u8 {
        match address {
            addr @ 0x000..=0x100 if !self.bootrom_disabled => {
                *self.bootrom.get(addr as usize).unwrap_or(&0xFF)
            }
            addr @ 0..=0x3FFF => *self.rom.get(addr as usize).unwrap_or(&0xFF),
            addr @ 0x4000..=0x7FFF => {
                let final_addr = addr + u16::from(self.rom_bank - 1) * 0x4000;
                *self.rom.get(final_addr as usize).unwrap_or(&0xFF)
            }
            0xFF50 => self.bootrom_disabled as u8,
            addr => 0xFF,
        }
    }

    fn write(&mut self, address: u16, val: u8) {
        match address {
            0x2000..=0x3FFF => {
                if val == 0 {
                    self.rom_bank = 1;
                } else {
                    self.rom_bank = val;
                }
            }
            addr @ 0x4000..=0x5FFF => println!("Write of {} to ram bank {}", val, addr),
            addr @ 0x6000..=0x7FFF => println!("Write of {} to bank sel {}", val, addr),
            0xFF50 => self.bootrom_disabled = val != 0,
            _ => {}
        }
    }
}

impl fmt::Display for MbcOne {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let header = header::Header::new(&self.rom);
        write!(f, "{}", header)
    }
}
