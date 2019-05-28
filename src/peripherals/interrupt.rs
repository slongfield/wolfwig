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

    const VBLANK_PC: u16 = 0x40;
    const LCD_STAT_PC: u16 = 0x48;
    const TIMER_PC: u16 = 0x50;
    const SERIAL_PC: u16 = 0x58;
    const JOYPAD_PC: u16 = 0x60;

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
            Self::FLAG => 0xE0 | self.flag,
            Self::ENABLE => 0xE0 | self.enable,
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

    /// Returns the pc for the highest prioirty interrupt that's enabled and whose flag is set,
    /// or None if no interrupts are ready.
    pub fn get_interrupt_pc(&self) -> Option<u16> {
        if (self.enable & Self::VBLANK != 0) && (self.flag & Self::VBLANK != 0) {
            return Some(Self::VBLANK_PC);
        }
        if (self.enable & Self::LCD_STAT != 0) && (self.flag & Self::LCD_STAT != 0) {
            return Some(Self::LCD_STAT_PC);
        }
        if (self.enable & Self::TIMER != 0) && (self.flag & Self::TIMER != 0) {
            return Some(Self::TIMER_PC);
        }
        if (self.enable & Self::SERIAL != 0) && (self.flag & Self::SERIAL != 0) {
            return Some(Self::SERIAL_PC);
        }
        if (self.enable & Self::JOYPAD != 0) && (self.flag & Self::JOYPAD != 0) {
            return Some(Self::JOYPAD_PC);
        }
        None
    }

    /// Clears the flag of the current higest-priority enabled interrupt.
    pub fn disable_interrupt(&mut self) {
        if (self.enable & Self::VBLANK != 0) && (self.flag & Self::VBLANK != 0) {
            self.set_flag(Irq::Vblank, false);
        } else if (self.enable & Self::LCD_STAT != 0) && (self.flag & Self::LCD_STAT != 0) {
            self.set_flag(Irq::LCDStat, false);
        } else if (self.enable & Self::TIMER != 0) && (self.flag & Self::TIMER != 0) {
            self.set_flag(Irq::Timer, false);
        } else if (self.enable & Self::SERIAL != 0) && (self.flag & Self::SERIAL != 0) {
            self.set_flag(Irq::Serial, false);
        } else if (self.enable & Self::JOYPAD != 0) && (self.flag & Self::JOYPAD != 0) {
            self.set_flag(Irq::Joypad, false);
        }
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
        assert_eq!(
            interrupt.get_interrupt_pc().unwrap(),
            Interrupt::LCD_STAT_PC
        );

        interrupt.set_enable(Irq::Vblank, true);
        assert_eq!(interrupt.get_interrupt_pc().unwrap(), Interrupt::VBLANK_PC);

        interrupt.set_flag(Irq::LCDStat, false);
        assert_eq!(interrupt.get_interrupt_pc().unwrap(), Interrupt::VBLANK_PC);

        interrupt.set_flag(Irq::Vblank, false);
        assert_eq!(interrupt.get_interrupt_pc().unwrap(), Interrupt::TIMER_PC);

        interrupt.set_flag(Irq::Timer, false);
        assert!(interrupt.get_interrupt_pc().is_none());
    }
}
