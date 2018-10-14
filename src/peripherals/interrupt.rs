///! Interrupt handler peripheral.

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Irq {
    Vblank,
    LCDStat,
    Timer,
    Serial,
    Joypad,
}

pub struct Interrupt {
    enable: u8,
    flag: u8,
}

impl Interrupt {
    const FLAG: u16 = 0xFF0F;
    const ENABLE: u16 = 0xFFFF;

    const VBLANK: u8 = 1;
    const LCD_STAT: u8 = 1 << 1;
    const TIMER: u8 = 1 << 2;
    const SERIAL: u8 = 1 << 3;
    const JOYPAD: u8 = 1 << 4;

    pub fn new() -> Self {
        Self { enable: 0, flag: 0 }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            Self::FLAG => self.flag = val,
            Self::ENABLE => self.enable = val,
            _ => panic!(
                "Attempted to write interrupt with unmapped addr: {:#x}",
                addr
            ),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            Self::FLAG => self.flag,
            Self::ENABLE => self.enable,
            _ => panic!(
                "Attempted to read interrupt with unmapped addr: {:#x}",
                addr
            ),
        }
    }

    pub fn set_flag(&mut self, i: Irq, val: bool) {
        match (i, val) {
            (Irq::Vblank, true) => self.flag |= Self::VBLANK,
            (Irq::Vblank, false) => self.flag &= !Self::VBLANK,
            (Irq::LCDStat, true) => self.flag |= Self::LCD_STAT,
            (Irq::LCDStat, false) => self.flag &= !Self::LCD_STAT,
            (Irq::Timer, true) => self.flag |= Self::TIMER,
            (Irq::Timer, false) => self.flag &= !Self::TIMER,
            (Irq::Serial, true) => self.flag |= Self::SERIAL,
            (Irq::Serial, false) => self.flag &= !Self::SERIAL,
            (Irq::Joypad, true) => self.flag |= Self::JOYPAD,
            (Irq::Joypad, false) => self.flag &= !Self::JOYPAD,
        }
    }

    pub fn set_enable(&mut self, i: Irq, val: bool) {
        match (i, val) {
            (Irq::Vblank, true) => self.enable |= Self::VBLANK,
            (Irq::Vblank, false) => self.enable &= !Self::VBLANK,
            (Irq::LCDStat, true) => self.enable |= Self::LCD_STAT,
            (Irq::LCDStat, false) => self.enable &= !Self::LCD_STAT,
            (Irq::Timer, true) => self.enable |= Self::TIMER,
            (Irq::Timer, false) => self.enable &= !Self::TIMER,
            (Irq::Serial, true) => self.enable |= Self::SERIAL,
            (Irq::Serial, false) => self.enable &= !Self::SERIAL,
            (Irq::Joypad, true) => self.enable |= Self::JOYPAD,
            (Irq::Joypad, false) => self.enable &= !Self::JOYPAD,
        }
    }

    /// Returns the highest prioirty that's enabled and whose flag is set, or None if no
    /// interrupts are ready.
    pub fn get_interrupt(&mut self) -> Option<Irq> {
        if (self.enable & Self::VBLANK != 0) && (self.flag & Self::VBLANK != 0) {
            return Some(Irq::Vblank);
        }
        if (self.enable & Self::LCD_STAT != 0) && (self.flag & Self::LCD_STAT != 0) {
            return Some(Irq::LCDStat);
        }
        if (self.enable & Self::TIMER != 0) && (self.flag & Self::TIMER != 0) {
            return Some(Irq::Timer);
        }
        if (self.enable & Self::SERIAL != 0) && (self.flag & Self::SERIAL != 0) {
            return Some(Irq::Serial);
        }
        if (self.enable & Self::JOYPAD != 0) && (self.flag & Self::JOYPAD != 0) {
            return Some(Irq::Joypad);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupt_enable() {
        let mut interrupt = Interrupt::new();

        interrupt.set_enable(Irq::Timer, true);
        interrupt.set_enable(Irq::LCDStat, true);
        interrupt.set_flag(Irq::Vblank, true);
        interrupt.set_flag(Irq::Timer, true);
        interrupt.set_flag(Irq::LCDStat, true);
        assert_eq!(interrupt.get_interrupt().unwrap(), Irq::LCDStat);

        interrupt.set_enable(Irq::Vblank, true);
        assert_eq!(interrupt.get_interrupt().unwrap(), Irq::Vblank);

        interrupt.set_flag(Irq::LCDStat, false);
        assert_eq!(interrupt.get_interrupt().unwrap(), Irq::Vblank);

        interrupt.set_flag(Irq::Vblank, false);
        assert_eq!(interrupt.get_interrupt().unwrap(), Irq::Timer);

        interrupt.set_flag(Irq::Timer, false);
        assert!(interrupt.get_interrupt().is_none());
    }
}
