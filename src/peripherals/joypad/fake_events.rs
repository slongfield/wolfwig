///! Fake event stream, for testing.
use peripherals::joypad::events::{EventHandler, State};
// TODO(slongfield): Add a back channel for injecting events.

pub struct FakeEvents {}

impl FakeEvents {
    pub fn new() -> Self {
        Self {}
    }
}

impl EventHandler for FakeEvents {
    fn get_state(&mut self) -> State {
        State::new()
    }
    fn clear_keydown(&mut self) {}
}
