use peripherals::interrupt::Interrupt;

// Note: This timer is based off of the DMG timer in the Cycle-Accurate GameBoy Docs v 0.0.X by
// AntonioND. It should accurate represent the bugs in the DMG timer, but not accurately represent
// the separate set of bugs in the CGB timer.
// TODO(slongfield): Make a CGB timer, and write a bunch of testroms.
#[derive(Debug)]
pub struct Timer {
    divider: u16,
    counter: u8,
    modulo: u8,
    start: bool,
    input_clock: u8,
    set_counter: bool,
    prev_increment_bit: bool,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            divider: 0,
            counter: 0,
            modulo: 0,
            start: false,
            input_clock: 0,
            prev_increment_bit: false,
            set_counter: false,
        }
    }

    pub fn step(&mut self, interrupt: &mut Interrupt) {
        self.divider = self.divider.wrapping_add(4);
        if self.set_counter {
            debug!("Setting off timer interrupt");
            self.set_counter = false;
            interrupt.set_timer_trigger(1);
            self.counter = self.modulo;
        }
        if self.start && self.increment_bit_set() && !self.prev_increment_bit {
            self.counter = self.counter.wrapping_add(1);
            if self.counter == 0 {
                self.set_counter = true;
            }
        }
        self.prev_increment_bit = self.increment_bit_set();
        if self.start {
            debug!("{:?}", self);
        }
    }

    pub fn set_divider(&mut self) {
        self.divider = 4;
    }

    pub fn set_counter(&mut self, val: u8) {
        self.counter = val;
    }

    pub fn set_modulo(&mut self, val: u8) {
        self.modulo = val;
    }

    pub fn set_start(&mut self, val: u8) {
        self.start = val != 0;
    }

    pub fn set_input_clock(&mut self, val: u8) {
        self.input_clock = val & 0x3;
    }

    pub fn divider(&self) -> u8 {
        (self.divider >> 8) as u8
    }

    pub fn counter(&self) -> u8 {
        self.counter
    }

    pub fn modulo(&self) -> u8 {
        self.modulo
    }

    pub fn start(&self) -> u8 {
        u8::from(self.start)
    }

    pub fn input_clock(&self) -> u8 {
        self.input_clock
    }

    fn increment_bit_set(&self) -> bool {
        let bit = match self.input_clock {
            0b00 => 10,
            0b01 => 4,
            0b10 => 6,
            0b11 => 8,
            _ => unreachable!(),
        };
        self.divider & (1 << bit) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_update_time_clock_00() {
        // The timer should update once every 1024 cycles (256 steps) after the div counter has
        // been reset.
        let mut timer = Timer::new();
        let mut irq = Interrupt::new();

        timer.set_divider();
        timer.set_modulo(0);
        timer.set_input_clock(0);
        timer.set_start(1);

        for _ in 0..511 {
            timer.step(&mut irq);
        }

        assert_eq!(timer.counter(), 1);
    }
}
