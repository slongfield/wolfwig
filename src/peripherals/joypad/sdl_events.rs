use sdl2::event::Event as SdlEvent;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;

use peripherals::joypad::events::{EventHandler, State};

pub struct SdlEvents {
    events: EventPump,
    state: State,
}

///! `EventHandler` for sdl
impl SdlEvents {
    pub fn new(events: EventPump) -> Self {
        Self {
            state: State::new(),
            events,
        }
    }
}

impl EventHandler for SdlEvents {
    // TODO(slongfield): This is still the root of performance problems.
    fn get_state(&mut self) -> State {
        for event in self.events.poll_iter() {
            // TODO(slongfield): Make this configurable. Currently optimized for my
            // Kinesis keyboard, but that's somewhat of an uncommon layout.
            match event {
                SdlEvent::Quit { .. } => {
                    self.state.shutdown = true;
                }
                SdlEvent::KeyDown {
                    keycode: Some(code),
                    ..
                } => {
                    let mut set_keydown = true;
                    debug!("Got keydown {:?}", code);
                    match code {
                        Keycode::Escape => self.state.shutdown = true,
                        Keycode::W => self.state.up = true,
                        Keycode::A => self.state.left = true,
                        Keycode::S => self.state.down = true,
                        Keycode::D => self.state.right = true,
                        Keycode::J => self.state.b = true,
                        Keycode::K => self.state.a = true,
                        Keycode::Backspace => self.state.select = true,
                        Keycode::Space => self.state.start = true,
                        _ => set_keydown = false,
                    }
                    if set_keydown {
                        self.state.keydown = true;
                    }
                }
                SdlEvent::KeyUp {
                    keycode: Some(code),
                    ..
                } => {
                    debug!("Got keyup {:?}", code);
                    match code {
                        Keycode::W => self.state.up = false,
                        Keycode::A => self.state.left = false,
                        Keycode::S => self.state.down = false,
                        Keycode::D => self.state.right = false,
                        Keycode::J => self.state.b = false,
                        Keycode::K => self.state.a = false,
                        Keycode::Backspace => self.state.select = false,
                        Keycode::Space => self.state.start = false,
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        self.state
    }

    fn clear_keydown(&mut self) {
        self.state.keydown = false;
    }
}
