///! Fake event stream, for testing.
use peripherals::joypad::events::{Event, EventStream};
// TODO(slongfield): Add a back channel for injecting events.

pub struct FakeEvents {}

impl FakeEvents {
    pub fn new() -> Self {
        Self {}
    }
}

impl EventStream for FakeEvents {
    fn next_event(&mut self) -> Option<Event> {
        None
    }
}
