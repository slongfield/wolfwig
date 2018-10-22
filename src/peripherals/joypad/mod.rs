///! Joypad is the joypad peripheral
use sdl2::EventPump;
use std::process;

mod events;
mod fake_events;
mod sdl_events;

pub struct Joypad {
    events: Box<events::EventHandler>,
    select_button: bool,
    select_direction: bool,
    reg: u8,
    counter: usize,
}

impl Joypad {
    const JOYP: u16 = 0xFF00;

    // How frequenlty to check for new updates, in cycles. This is a janky hack, needed
    // because the
    // SDL event polling can't be moved to a different thread, and is kind of slow.
    // TODO(slongfield): Figure out a beter solution.
    const UPDATE_INTERVAL: usize = 1000;

    pub fn new_sdl(events: EventPump) -> Self {
        let events = Box::new(sdl_events::SdlEvents::new(events));
        Self {
            events,
            select_button: false,
            select_direction: false,
            reg: 0xF,
            counter: 0,
        }
    }

    pub fn new_fake() -> Self {
        let events = Box::new(fake_events::FakeEvents::new());
        Self {
            events,
            select_button: false,
            select_direction: false,
            reg: 0xF,
            counter: 0,
        }
    }

    pub fn step(&mut self) {
        self.counter += 1;
        if self.counter == Self::UPDATE_INTERVAL {
            if self.events.get_state().keydown {}
            let state = self.events.get_state();

            if state.shutdown {
                process::exit(0);
            }

            if state.keydown {
                // TODO(slongfield): Set interrupt.
            }

            // All of the joypad signals are active-low.
            self.reg = 0xFF;
            self.reg ^= u8::from(self.select_button) << 5;
            self.reg ^= u8::from(self.select_direction) << 4;

            // TODO(slongfield): What happens if both button and direction are enabled at
            // the same time?
            if self.select_button {
                self.reg ^= u8::from(state.start) << 3;
                self.reg ^= u8::from(state.select) << 2;
                self.reg ^= u8::from(state.b) << 1;
                self.reg ^= u8::from(state.a);
            }
            if self.select_direction {
                self.reg ^= u8::from(state.down) << 3;
                self.reg ^= u8::from(state.up) << 2;
                self.reg ^= u8::from(state.left) << 1;
                self.reg ^= u8::from(state.right);
            }
            self.events.clear_keydown();
            self.counter = 0;
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            Self::JOYP => {
                // Active-low signals.
                self.select_button = val & (1 << 5) == 0;
                self.select_direction = val & (1 << 4) == 0;
            }
            addr => panic!("Attempted to write joypad with unmapped addr: {:#x}", addr),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            Self::JOYP => self.reg,
            addr => panic!("Attempted to write joypad with unmapped addr: {:#x}", addr),
        }
    }
}
