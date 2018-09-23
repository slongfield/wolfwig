use mem::header;

pub struct Memory {
    pub header: header::Header,
    // TODO(slongfield): Support special boot ROM.
    // TODO(slongfield): Handle different memory controllers, and ROM banking.
    // 0x0000-3FFF for bank 0, 0x4000-7FFF for switchable bank.
    pub rom: Vec<u8>,
    // TODO(slongfield): Switchable banks.
    // Ox8000-0x9FFF
    vram: [u8; 0x2000],
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
    // Sprite attribute table.
    // 0xFE00-0xFE9F
    oam: [u8; 0x100],
    // IO registers, 0xFF00-0xFF7F
    io_regs: [u8; 0x80],
    // High RAM. 0xFF80-0xFFFE
    // TODO(slongfield): Unclear what this is used for?
    high_ram: [u8; 0x17f],
    // Interrupts enable register, 0xFFF.
    interrupt_enable: u8,
}

impl Memory {
    pub fn new(rom: Vec<u8>) -> Memory {
        let header = header::Header::new(&rom);
        Memory {
            header,
            rom,
            vram: [0; 0x2000],
            xram: [0; 0x2000],
            wram0: [0; 0x1000],
            wram1_n: [0; 0x1000],
            oam: [0; 0x100],
            io_regs: [0; 0x80],
            high_ram: [0; 0x17f],
            interrupt_enable: 0,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            addr @ 0x0000..=0x7FFF => self.rom[addr as usize],
            addr @ 0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize],
            addr @ 0xA000..=0xBFFF => self.xram[(addr - 0xA000) as usize],
            addr @ 0xC000..=0xCFFF => self.wram0[(addr - 0xC000) as usize],
            addr @ 0xD000..=0xDFFF => self.wram1_n[(addr - 0xD000) as usize],
            addr @ 0xE000..=0xFDFF => {
                // Echo RAM, maps back onto 0xC000-0XDDFF
                self.read(addr - 0x2000)
            }
            addr @ 0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize],
            addr @ 0xFEA0..=0xFEFF => {
                // Reads here return 0 on DMG, random data on GBC, Pan docs say they usualy
                // don't happen, so log if they do, since that could be interesting.
                info!("Read from unmapped memory region: {}", addr);
                0
            }
            addr @ 0xFF00..=0xFF7F => self.io_regs[(addr - 0xFF00) as usize],
            addr @ 0xFF80..=0xFFFE => self.high_ram[(addr - 0xFF80) as usize],
            0xFFFF => self.interrupt_enable,
            bad_addr => panic!("Attempted to read from unmapped address: {}!", bad_addr),
        }
    }

    pub fn write(&mut self, address: u16, val: u8) {
        match address {
            addr @ 0x0000..=0x7FFF => {}
            addr @ 0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize] = val,
            addr @ 0xA000..=0xBFFF => self.xram[(addr - 0xA000) as usize] = val,
            addr @ 0xC000..=0xCFFF => self.wram0[(addr - 0xC000) as usize] = val,
            addr @ 0xD000..=0xDFFF => self.wram1_n[(addr - 0xD000) as usize] = val,
            addr @ 0xE000..=0xFDFF => {
                // Echo RAM, maps back onto 0xC000-0XDDFF
                self.write(addr - 0x2000, val)
            }
            addr @ 0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize] = val,
            addr @ 0xFEA0..=0xFEFF => {
                // Writes here are ignored, Pan docs say they usualy don't happen, so log if they
                // do, since that could be interesting.
                info!("Write to unmapped memory region: {}", addr)
            }
            addr @ 0xFF00..=0xFF7F => self.io_regs[(addr - 0xFF00) as usize] = val,
            addr @ 0xFF80..=0xFFFE => self.high_ram[(addr - 0xFF80) as usize] = val,
            0xFFFF => self.interrupt_enable = val,
            bad_addr => panic!("Attempted to read from unmapped address: {}!", bad_addr),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_after_write_ram() {
        let mut mem = Memory::new(vec![0; 0x1000]);
        mem.write(0xC042, 41);
        assert_eq!(mem.read(0xC042), 41);
    }

    #[test]
    fn read_after_write_shadow_ram() {
        let mut mem = Memory::new(vec![0; 0x1000]);
        mem.write(0xE042, 17);
        assert_eq!(mem.read(0xC042), 17);
    }

}