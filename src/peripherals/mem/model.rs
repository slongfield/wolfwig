use peripherals::mem::header;

pub struct Memory {
    pub header: header::Header,
    // TODO(slongfield): Handle different memory controllers, and ROM banking.
    // 0x0000-3FFF for bank 0, 0x4000-7FFF for switchable bank.
    // Reads from 0x0000 -> 0x0100 will read from the bootrom until 1 has been written to 0xFF50
    bootrom: Vec<u8>,
    rom: Vec<u8>,
    bootrom_disabled: bool,
    // External RAM, in cartrige, may be switchable?
    // 0xA000-0xBFFF
    xram: [u8; 0x2000],
    // Working RAM bank 0
    // 0xC000-0xCFFF,
    wram0: [u8; 0x1000],
    // TODO(slongfield): Switchable banks.
    // Bank 1 in DMG mode, banks 1-7 in CGB mode
    // 0xD000-0xDFFF
    wram1_n: [u8; 0x1000],
    // High RAM. 0xFF80-0xFFFE
    high_ram: [u8; 0x17f],
}

impl Memory {
    pub fn new(bootrom: Vec<u8>, rom: Vec<u8>) -> Self {
        let header = header::Header::new(&rom);
        Self {
            header,
            bootrom,
            rom,
            bootrom_disabled: false,
            xram: [0; 0x2000],
            wram0: [0; 0x1000],
            wram1_n: [0; 0x1000],
            high_ram: [0; 0x17f],
        }
    }

    pub fn write(&mut self, address: u16, val: u8) {
        let address = address as usize;
        match address {
            0x0000..=0x7FFF => {}
            addr @ 0xA000..=0xBFFF => self.xram[addr - 0xA000] = val,
            addr @ 0xC000..=0xCFFF => self.wram0[addr - 0xC000] = val,
            addr @ 0xD000..=0xDFFF => self.wram1_n[addr - 0xD000] = val,
            addr @ 0xE000..=0xFDFF => self.write((addr - 0x2000) as u16, val),
            addr @ 0xFEA0..=0xFEFF => info!("Write to unmapped memory region: {}", addr),
            0xFF50 => self.bootrom_disabled = val != 0,
            addr @ 0xFF80..=0xFFFE => self.high_ram[addr - 0xFF80] = val,
            addr => panic!(
                "Attempted to read mem from unmapped address: {:#04X}!",
                addr
            ),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        let address = address as usize;
        match address {
            addr @ 0x0000..=0x00FF if !self.bootrom_disabled => self.bootrom[addr],
            addr @ 0x0000..=0x7FFF => self.rom[addr],
            addr @ 0xA000..=0xBFFF => self.xram[addr - 0xA000],
            addr @ 0xC000..=0xCFFF => self.wram0[addr - 0xC000],
            addr @ 0xD000..=0xDFFF => self.wram1_n[addr - 0xD000],
            0xFF50 => self.bootrom_disabled as u8,
            addr @ 0xFF80..=0xFFFE => self.high_ram[addr - 0xFF80],
            addr => panic!(
                "Attempted to read mem from unmapped address: {:#04X}!",
                addr
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_after_write_ram() {
        let mut mem = Memory::new(vec![0; 0x100], vec![0; 0x1000]);
        mem.write(0xC042, 41);
        assert_eq!(mem.read(0xC042), 41);
    }

    #[test]
    fn read_after_write_shadow_ram() {
        let mut mem = Memory::new(vec![0; 0x100], vec![0; 0x1000]);
        mem.write(0xE042, 17);
        assert_eq!(mem.read(0xC042), 17);
    }

}
