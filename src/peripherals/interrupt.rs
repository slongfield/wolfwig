///! Interrupt handler peripheral.

struct Flag {
    enable: bool,
    trigger: bool,
}

impl Flag {
    pub fn new() -> Self {
        Self {
            enable: false,
            trigger: false,
        }
    }
}

pub struct Interrupt {
    vblank: Flag,
    lcd_stat: Flag,
    timer: Flag,
    serial: Flag,
    joypad: Flag,
    unused: u8,
}

impl Interrupt {
    const VBLANK_PC: u16 = 0x40;
    const LCD_STAT_PC: u16 = 0x48;
    const TIMER_PC: u16 = 0x50;
    const SERIAL_PC: u16 = 0x58;
    const JOYPAD_PC: u16 = 0x60;

    pub fn new() -> Self {
        Self {
            vblank: Flag::new(),
            lcd_stat: Flag::new(),
            timer: Flag::new(),
            serial: Flag::new(),
            joypad: Flag::new(),
            unused: 0,
        }
    }

    pub fn set_vblank_enable(&mut self, val: u8) {
        self.vblank.enable = val != 0;
    }

    pub fn set_vblank_trigger(&mut self, val: u8) {
        self.vblank.trigger = val != 0;
    }

    pub fn vblank_enable(&self) -> bool {
        self.vblank.enable
    }

    pub fn vblank_trigger(&self) -> bool {
        self.vblank.trigger
    }

    pub fn set_lcd_stat_enable(&mut self, val: u8) {
        self.lcd_stat.enable = val != 0;
    }

    pub fn set_lcd_stat_trigger(&mut self, val: u8) {
        self.lcd_stat.trigger = val != 0;
    }

    pub fn lcd_stat_enable(&self) -> bool {
        self.lcd_stat.enable
    }

    pub fn lcd_stat_trigger(&self) -> bool {
        self.lcd_stat.trigger
    }

    pub fn set_timer_enable(&mut self, val: u8) {
        self.timer.enable = val != 0;
    }

    pub fn set_timer_trigger(&mut self, val: u8) {
        self.timer.trigger = val != 0;
    }

    pub fn timer_enable(&self) -> bool {
        self.timer.enable
    }

    pub fn timer_trigger(&self) -> bool {
        self.timer.trigger
    }

    pub fn set_serial_enable(&mut self, val: u8) {
        self.serial.enable = val != 0;
    }

    pub fn set_serial_trigger(&mut self, val: u8) {
        self.serial.trigger = val != 0;
    }

    pub fn serial_enable(&self) -> bool {
        self.serial.enable
    }

    pub fn serial_trigger(&self) -> bool {
        self.serial.trigger
    }

    pub fn set_joypad_enable(&mut self, val: u8) {
        self.joypad.enable = val != 0;
    }

    pub fn set_joypad_trigger(&mut self, val: u8) {
        self.joypad.trigger = val != 0;
    }

    pub fn joypad_enable(&self) -> bool {
        self.joypad.enable
    }

    pub fn joypad_trigger(&self) -> bool {
        self.joypad.trigger
    }

    pub fn set_unused(&mut self, val: u8) {
        self.unused = val
    }

    pub fn unused(&self) -> u8 {
        self.unused
    }

    /// Returns the pc for the highest prioirty interrupt that's enabled and whose flag is set,
    /// or None if no interrupts are ready.
    pub fn get_interrupt_pc(&self) -> Option<u16> {
        if self.vblank.enable && self.vblank.trigger {
            return Some(Self::VBLANK_PC);
        }
        if self.lcd_stat.enable && self.lcd_stat.trigger {
            return Some(Self::LCD_STAT_PC);
        }
        if self.timer.enable && self.timer.trigger {
            return Some(Self::TIMER_PC);
        }
        if self.serial.enable && self.serial.trigger {
            return Some(Self::SERIAL_PC);
        }
        if self.joypad.enable && self.joypad.trigger {
            return Some(Self::JOYPAD_PC);
        }
        None
    }

    /// Clears the flag of the current higest-priority enabled interrupt.
    pub fn disable_interrupt(&mut self) {
        if self.vblank.enable && self.vblank.trigger {
            self.vblank.trigger = false;
        } else if self.lcd_stat.enable && self.lcd_stat.trigger {
            self.lcd_stat.trigger = false;
        } else if self.timer.enable && self.timer.trigger {
            self.timer.trigger = false;
        } else if self.serial.enable && self.serial.trigger {
            self.serial.trigger = false;
        } else if self.joypad.enable && self.joypad.trigger {
            self.joypad.trigger = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupt_enable() {
        let mut interrupt = Interrupt::new();

        interrupt.set_timer_enable(1);
        interrupt.set_lcd_stat_enable(1);
        interrupt.set_vblank_trigger(1);
        interrupt.set_timer_trigger(1);
        interrupt.set_lcd_stat_trigger(1);
        assert_eq!(
            interrupt.get_interrupt_pc().unwrap(),
            Interrupt::LCD_STAT_PC
        );

        interrupt.set_vblank_enable(1);
        assert_eq!(interrupt.get_interrupt_pc().unwrap(), Interrupt::VBLANK_PC);

        interrupt.set_lcd_stat_trigger(0);
        assert_eq!(interrupt.get_interrupt_pc().unwrap(), Interrupt::VBLANK_PC);

        interrupt.set_vblank_trigger(0);
        assert_eq!(interrupt.get_interrupt_pc().unwrap(), Interrupt::TIMER_PC);

        interrupt.set_timer_trigger(0);
        assert!(interrupt.get_interrupt_pc().is_none());
    }
}
