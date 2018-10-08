///! Joypad is the joypad peripheral
use mem::model::Memory;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use std::process;

pub struct Joypad {
    events: EventPump,
}

impl Joypad {
    pub fn new(events: EventPump) -> Self {
        Self { events }
    }

    pub fn step(&mut self, _mem: &mut Memory) {
        for event in self.events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => process::exit(0),
                _ => {}
            }
        }
    }
}
