use peripherals::interrupt::{Interrupt, Irq};

// Note: This timer is based off of the DMG timer in the Cycle-Accurate GameBoy Docs v 0.0.X by
// AntonioND. It should accurate represent the bugs in the DMG timer, but not accurately represent
// the separate set of bugs in the CGB timer.
// TODO(slongfield): Make a CGB timer, and write a bunch of testroms.
pub struct Timer {
    divider: u16,
    counter: u8,
    modulo: u8,
    control: u8,
    set_counter: bool,
    prev_increment_bit: bool,
}

impl Timer {
    const DIV: u16 = 0xFF04;
    const TIMA: u16 = 0xFF05;
    const TMA: u16 = 0xFF06;
    const TAC: u16 = 0xFF07;

    pub fn new() -> Self {
        Self {
            divider: 0,
            counter: 0,
            modulo: 0,
            control: 0,
            prev_increment_bit: false,
            set_counter: false,
        }
    }

    pub fn step(&mut self, interrupt: &mut Interrupt) {
        if self.set_counter {
            self.counter = self.modulo;
            interrupt.set_flag(Irq::Timer, true);
        }
        if self.enabled() && self.increment_bit_unset() && self.prev_increment_bit {
            self.counter = self.counter.wrapping_add(1);
            if self.counter == 0 {
                self.set_counter = true;
            }
        }
        self.prev_increment_bit = !self.increment_bit_unset();
        self.divider = self.divider.wrapping_add(4);
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            Self::DIV => self.divider = 0,
            Self::TIMA => self.counter = val,
            Self::TMA => self.modulo = val,
            Self::TAC => self.control = val,
            addr => panic!("Attempted to write Timer with unmapped addr: {:#x}", addr),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            Self::DIV => {
                println!(
                    "divider: {:#04x} counter: {:02x} modulo: {:02x} control: {:02x} enabled: {}",
                    self.divider,
                    self.counter,
                    self.modulo,
                    self.control,
                    self.enabled()
                );
                (self.divider >> 8) as u8
            }
            Self::TIMA => self.counter,
            Self::TMA => self.modulo,
            Self::TAC => self.control,
            addr => panic!("Attempted to read Timer with unmapped addr: {:#x}", addr),
        }
    }

    fn increment_bit_unset(&self) -> bool {
        let bit = match self.control & 0x3 {
            0b00 => 8,
            0b01 => 2,
            0b10 => 4,
            0b11 => 6,
            _ => unreachable!(),
        };
        self.divider & (1 << bit) == 0
    }

    fn enabled(&self) -> bool {
        self.control & (1 << 2) != 0
    }
}
