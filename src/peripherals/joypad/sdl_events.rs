///! EventStream for sdl
use sdl2::event::Event as SdlEvent;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;

use peripherals::joypad::events::{Event, EventStream};

pub struct SdlEvents {
    events: EventPump,
}

impl SdlEvents {
    pub fn new(events: EventPump) -> Self {
        Self { events }
    }
}

impl EventStream for SdlEvents {
    fn next_event(&mut self) -> Option<Event> {
        if let Some(event) = self.events.poll_event() {
            match event {
                SdlEvent::Quit { .. }
                | SdlEvent::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => Some(Event::PowerOff),
                _ => Some(Event::Ignore),
            }
        } else {
            None
        }
    }
}
