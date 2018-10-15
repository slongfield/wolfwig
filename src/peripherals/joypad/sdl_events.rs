///! `EventStream` for sdl
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

// TODO(slongfield): Make this configurable. Currently optimized for my Kinesis keyboard, but
// that's somewhat of an uncommon layout.
impl EventStream for SdlEvents {
    fn next_event(&mut self) -> Option<Event> {
        if let Some(event) = self.events.poll_event() {
            match event {
                SdlEvent::Quit { .. } => Some(Event::PowerOff),
                SdlEvent::KeyDown {
                    keycode: Some(code),
                    ..
                } => match code {
                    Keycode::Escape => Some(Event::PowerOff),
                    Keycode::W => Some(Event::Up(true)),
                    Keycode::A => Some(Event::Left(true)),
                    Keycode::S => Some(Event::Down(true)),
                    Keycode::D => Some(Event::Right(true)),
                    Keycode::J => Some(Event::B(true)),
                    Keycode::K => Some(Event::A(true)),
                    Keycode::Backspace => Some(Event::Select(true)),
                    Keycode::Space => Some(Event::Start(true)),
                    _ => Some(Event::Ignore),
                },
                SdlEvent::KeyUp {
                    keycode: Some(code),
                    ..
                } => match code {
                    Keycode::W => Some(Event::Up(false)),
                    Keycode::A => Some(Event::Left(false)),
                    Keycode::S => Some(Event::Down(false)),
                    Keycode::D => Some(Event::Right(false)),
                    Keycode::J => Some(Event::B(false)),
                    Keycode::K => Some(Event::A(false)),
                    Keycode::Backspace => Some(Event::Select(false)),
                    Keycode::Space => Some(Event::Start(false)),
                    _ => Some(Event::Ignore),
                },

                _ => Some(Event::Ignore),
            }
        } else {
            None
        }
    }
}
