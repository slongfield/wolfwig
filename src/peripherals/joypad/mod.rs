///! Joypad is the joypad peripheral
use sdl2::EventPump;
use std::process;

mod events;
mod fake_events;
mod sdl_events;

use peripherals::joypad::events::Event;

pub struct Joypad {
    events: Box<events::EventStream>,
}

impl Joypad {
    pub fn new_sdl(events: EventPump) -> Self {
        let events = Box::new(sdl_events::SdlEvents::new(events));
        Self { events }
    }

    pub fn new_fake() -> Self {
        let events = Box::new(fake_events::FakeEvents::new());
        Self { events }
    }

    pub fn step(&mut self) {
        for event in self.events.as_mut() {
            match event {
                Event::PowerOff => process::exit(0),
                _ => {}
            }
        }
    }

    pub fn write(&mut self, _addr: u16, _val: u8) {}

    pub fn read(&self, _addr: u16) -> u8 {
        0xF
    }
}
