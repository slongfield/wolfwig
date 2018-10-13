use mem::model::Memory;

// Note: This timer is based off of the DMG timer in the Cycle-Accurate GameBoy Docs v 0.0.X by
// AntonioND. It should accurate represent the bugs in the DMG timer, but not accurately represent
// the separate set of bugs in the CGB timer.
// TODO(slongfield): Make a CGB timer, and write a bunch of testroms.
pub struct Timer {
    divider: u16,
    counter: u8,
    modulo: u8,
    control: u8,
    // Set timer to 'modulo' value at the beginning of the next cycle.
    set_counter: Option<u8>,
    // If this changes from true to false, the counter is incremented.
    prev_increment_bit: bool,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            divider: 0,
            counter: 0,
            modulo: 0,
            control: 0,
            prev_increment_bit: false,
            set_counter: None,
        }
    }

    pub fn step(&mut self, mem: &mut Memory) {
        self.divider.wrapping_add(4);
        if self.set_counter {
            self.counter = self.modulo;
            self.set_interrupt_flag(mem);
        }
        if self.enabled() && self.increment_bit_unset() && self.prev_increment_bit {
            self.counter.wrapping_add(1);
            if self.counter == 0 {
                self.set_counter = true;
            }
        }
        self.prev_increment_bit = !self.increment_bit_unset();
    }

    pub fn clear_divdier(&mut self) {
        self.divider = 0;
    }

    pub fn get_divider(&self) -> u8 {
        (self.divider >> 8) as u8
    }

    pub fn set_counter(&mut self, val: u8) {
        self.counter = val;
    }

    pub fn get_counter(&self) -> u8 {
        self.counter
    }

    pub fn set_modulo(&mut self, val: u8) {
        self.modulo = val;
    }

    pub fn get_modulo(&self) -> u8 {
        self.modulo
    }

    pub fn set_control(&mut self, val: u8) {
        self.control = val;
    }

    pub fn get_control(&self) -> u8 {
        self.control
    }

    // TODO(slongfield): What happens if the timer is disabled when an interrupt should be
    // triggered on the next cycle?
    // TODO(slongfield): Writes to the interrupt vector should overwrite the timer write to the
    // interrupt vector. Test to make sure that's happening properly.
    fn set_interrupt_flag(&self, mem: &mut Memory) {
        mem.set_interrupt_flag(Interrupt::Timer, true);
    }

    fn increment_bit_unset(&self) -> u8 {
        let bit = match (self.control & 0x3) {
            0b00 => 9,
            0b01 => 3,
            0b10 => 5,
            0b11 => 7,
            _ => unreachable!(),
        };
        self.divider & (1 << bit) != 0
    }

    fn enabled(&self) -> bool {
        self.control & (1 << 3) != 0
    }
}
