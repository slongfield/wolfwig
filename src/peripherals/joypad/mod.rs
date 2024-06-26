///! Joypad is the joypad peripheral
use peripherals::interrupt::Interrupt;
use sdl2::EventPump;
use std::process;

mod events;
mod fake_events;
mod sdl_events;

pub struct Joypad {
    events: Box<events::EventHandler>,
    select_button: bool,
    select_direction: bool,
    state: u8,
    counter: usize,
}

impl Joypad {
    // How frequenlty to check for new updates, in cycles. This is a janky hack, needed
    // because the
    // SDL event polling can't be moved to a different thread, and is kind of slow.
    // TODO(slongfield): Figure out a beter solution. Maybe move _all_ of the SDL
    // stuff into a separate thread?
    const UPDATE_INTERVAL: usize = 100;

    pub fn new_sdl(events: EventPump) -> Self {
        let events = Box::new(sdl_events::SdlEvents::new(events));
        Self {
            events,
            select_button: true,
            select_direction: true,
            state: 0xF,
            counter: 0,
        }
    }

    pub fn new_fake() -> Self {
        let events = Box::new(fake_events::FakeEvents::new());
        Self {
            events,
            select_button: true,
            select_direction: true,
            state: 0xF,
            counter: 0,
        }
    }

    pub fn step(&mut self, interrupt: &mut Interrupt) {
        self.counter += 1;
        if self.counter == Self::UPDATE_INTERVAL {
            debug!("Updating state.");
            self.update(interrupt);
        }
    }

    pub fn set_select_direction(&mut self, val: u8) {
        debug!("Setting select direction to {}", val);
        self.select_direction = val != 0
    }

    pub fn set_select_button(&mut self, val: u8) {
        debug!("Setting select button to {}", val);
        self.select_button = val != 0
    }

    pub fn select_direction(&self) -> bool {
        self.select_direction
    }

    pub fn select_button(&self) -> bool {
        self.select_button
    }

    pub fn state(&self) -> u8 {
        self.state
    }

    pub fn update(&mut self, interrupt: &mut Interrupt) {
        if self.events.get_state().keydown {}
        let state = self.events.get_state();

        if state.shutdown {
            process::exit(0);
        }

        if state.keydown {
            interrupt.set_joypad_trigger(1);
        }

        self.state = 0;
        if !self.select_direction {
            self.state |= u8::from(state.down) << 3;
            self.state |= u8::from(state.up) << 2;
            self.state |= u8::from(state.left) << 1;
            self.state |= u8::from(state.right);
        }
        if !self.select_button {
            self.state |= u8::from(state.start) << 3;
            self.state |= u8::from(state.select) << 2;
            self.state |= u8::from(state.b) << 1;
            self.state |= u8::from(state.a);
        }
        // It's active low, so invert
        self.state = !self.state;
        self.events.clear_keydown();
        self.counter = 0;
    }
}
