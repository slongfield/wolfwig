use mem::header;

pub struct Memory {
    pub header: header::Header,
    // TODO(slongfield): Handle different memory controllers, and ROM banking.
    // 0x0000-3FFF for bank 0, 0x4000-7FFF for switchable bank.
    // Reads from 0x0000 -> 0x0100 will read from the bootrom until 1 has been written to 0xFF50
    bootrom: Vec<u8>,
    rom: Vec<u8>,
    bootrom_disabled: bool,
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
    high_ram: [u8; 0x17f],
    // Interrupts enable register, 0xFFF.
    interrupt_enable: u8,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Interrupt {
    Vblank,
    LCDStat,
    Timer,
    Serial,
    Joypad,
}

const VBLANK_BIT: u8 = 1;
const LCD_STAT_BIT: u8 = 1 << 1;
const TIMER_BIT: u8 = 1 << 2;
const SERIAL_BIT: u8 = 1 << 3;
const JOYPAD_BIT: u8 = 1 << 4;

impl Memory {
    pub fn new(bootrom: Vec<u8>, rom: Vec<u8>) -> Memory {
        let header = header::Header::new(&rom);
        Memory {
            header,
            bootrom,
            rom,
            bootrom_disabled: false,
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

    pub fn read(&self, address: usize) -> u8 {
        match address {
            addr @ 0x0000..=0x00FF if !self.bootrom_disabled => self.bootrom[addr],
            addr @ 0x0000..=0x7FFF => self.rom[addr],
            addr @ 0x8000..=0x9FFF => self.vram[addr - 0x8000],
            addr @ 0xA000..=0xBFFF => self.xram[addr - 0xA000],
            addr @ 0xC000..=0xCFFF => self.wram0[addr - 0xC000],
            addr @ 0xD000..=0xDFFF => self.wram1_n[addr - 0xD000],
            addr @ 0xE000..=0xFDFF => {
                // Echo RAM, maps back onto 0xC000-0XDDFF
                self.read(addr - 0x2000)
            }
            addr @ 0xFE00..=0xFE9F => self.oam[addr - 0xFE00],
            addr @ 0xFEA0..=0xFEFF => {
                // Reads here return 0 on DMG, random data on GBC, Pan docs say they usualy
                // don't happen, so log if they do, since that could be interesting.
                info!("Read from unmapped memory region: {}", addr);
                0
            }
            addr @ 0xFF00..=0xFF7F => self.io_regs[addr - 0xFF00],
            addr @ 0xFF80..=0xFFFE => self.high_ram[addr - 0xFF80],
            0xFFFF => self.interrupt_enable,
            bad_addr => panic!("Attempted to read from unmapped address: {}!", bad_addr),
        }
    }

    pub fn write(&mut self, address: usize, val: u8) {
        match address {
            0x0000..=0x7FFF => {}
            addr @ 0x8000..=0x9FFF => self.vram[addr - 0x8000] = val,
            addr @ 0xA000..=0xBFFF => self.xram[addr - 0xA000] = val,
            addr @ 0xC000..=0xCFFF => self.wram0[addr - 0xC000] = val,
            addr @ 0xD000..=0xDFFF => self.wram1_n[addr - 0xD000] = val,
            addr @ 0xE000..=0xFDFF => {
                // Echo RAM, maps back onto 0xC000-0XDDFF
                self.write(addr - 0x2000, val)
            }
            addr @ 0xFE00..=0xFE9F => self.oam[addr - 0xFE00] = val,
            addr @ 0xFEA0..=0xFEFF => {
                // Writes here are ignored, Pan docs say they usualy don't happen, so log if they
                // do, since that could be interesting.
                info!("Write to unmapped memory region: {}", addr)
            }
            addr @ 0xFF00..=0xFF7F => self.io_regs[addr - 0xFF00] = val,
            addr @ 0xFF80..=0xFFFE => self.high_ram[addr - 0xFF80] = val,
            0xFFFF => self.interrupt_enable = val,
            bad_addr => panic!("Attempted to read from unmapped address: {}!", bad_addr),
        }
    }

    pub fn set_interrupt_flag(&mut self, i: Interrupt, val: bool) {
        match (i, val) {
            (Interrupt::Vblank, true) => self.io_regs[0xF] |= VBLANK_BIT,
            (Interrupt::Vblank, false) => self.io_regs[0xF] &= !VBLANK_BIT,
            (Interrupt::LCDStat, true) => self.io_regs[0xF] |= LCD_STAT_BIT,
            (Interrupt::LCDStat, false) => self.io_regs[0xF] &= !LCD_STAT_BIT,
            (Interrupt::Timer, true) => self.io_regs[0xF] |= TIMER_BIT,
            (Interrupt::Timer, false) => self.io_regs[0xF] &= !TIMER_BIT,
            (Interrupt::Serial, true) => self.io_regs[0xF] |= SERIAL_BIT,
            (Interrupt::Serial, false) => self.io_regs[0xF] &= !SERIAL_BIT,
            (Interrupt::Joypad, true) => self.io_regs[0xF] |= JOYPAD_BIT,
            (Interrupt::Joypad, false) => self.io_regs[0xF] &= !JOYPAD_BIT,
        }
    }

    pub fn set_interrupt_enable(&mut self, i: Interrupt, val: bool) {
        match (i, val) {
            (Interrupt::Vblank, true) => self.interrupt_enable |= VBLANK_BIT,
            (Interrupt::Vblank, false) => self.interrupt_enable &= !VBLANK_BIT,
            (Interrupt::LCDStat, true) => self.interrupt_enable |= LCD_STAT_BIT,
            (Interrupt::LCDStat, false) => self.interrupt_enable &= !LCD_STAT_BIT,
            (Interrupt::Timer, true) => self.interrupt_enable |= TIMER_BIT,
            (Interrupt::Timer, false) => self.interrupt_enable &= !TIMER_BIT,
            (Interrupt::Serial, true) => self.interrupt_enable |= SERIAL_BIT,
            (Interrupt::Serial, false) => self.interrupt_enable &= !SERIAL_BIT,
            (Interrupt::Joypad, true) => self.interrupt_enable |= JOYPAD_BIT,
            (Interrupt::Joypad, false) => self.interrupt_enable &= !JOYPAD_BIT,
        }
    }

    /// Returns the highest prioirty that's enabled and whose flag is set, or None if no
    /// interrupts are ready.
    pub fn get_interrupt(&mut self) -> Option<Interrupt> {
        if (self.interrupt_enable & VBLANK_BIT != 0) && (self.io_regs[0xF] & VBLANK_BIT != 0) {
            return Some(Interrupt::Vblank);
        }
        if (self.interrupt_enable & LCD_STAT_BIT != 0) && (self.io_regs[0xF] & LCD_STAT_BIT != 0) {
            return Some(Interrupt::LCDStat);
        }
        if (self.interrupt_enable & TIMER_BIT != 0) && (self.io_regs[0xF] & TIMER_BIT != 0) {
            return Some(Interrupt::Timer);
        }
        if (self.interrupt_enable & SERIAL_BIT != 0) && (self.io_regs[0xF] & SERIAL_BIT != 0) {
            return Some(Interrupt::Serial);
        }
        if (self.interrupt_enable & JOYPAD_BIT != 0) && (self.io_regs[0xF] & JOYPAD_BIT != 0) {
            return Some(Interrupt::Joypad);
        }
        None
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

    #[test]
    fn interrupt_enable() {
        let mut mem = Memory::new(vec![0; 0x100], vec![0; 0x1000]);
        mem.set_interrupt_enable(Interrupt::Timer, true);
        mem.set_interrupt_enable(Interrupt::LCDStat, true);
        mem.set_interrupt_flag(Interrupt::Vblank, true);
        mem.set_interrupt_flag(Interrupt::Timer, true);
        mem.set_interrupt_flag(Interrupt::LCDStat, true);
        assert_eq!(mem.get_interrupt().unwrap(), Interrupt::LCDStat);

        mem.set_interrupt_enable(Interrupt::Vblank, true);
        assert_eq!(mem.get_interrupt().unwrap(), Interrupt::Vblank);

        mem.set_interrupt_flag(Interrupt::LCDStat, false);
        assert_eq!(mem.get_interrupt().unwrap(), Interrupt::Vblank);

        mem.set_interrupt_flag(Interrupt::Vblank, false);
        assert_eq!(mem.get_interrupt().unwrap(), Interrupt::Timer);

        mem.set_interrupt_flag(Interrupt::Timer, false);
        assert!(mem.get_interrupt().is_none());
    }

}
