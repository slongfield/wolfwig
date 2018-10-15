///! Interface that needs to be implemented to create a `Joypad`

///! Events that can be generated from the stream. For events that have a boolean, true means
///! pressed, and false means released.
pub enum Event {
    PowerOff,
    Start(bool),
    Select(bool),
    A(bool),
    B(bool),
    Up(bool),
    Down(bool),
    Left(bool),
    Right(bool),
    Ignore,
}

pub trait EventStream {
    fn next_event(&mut self) -> Option<Event>;
}

impl Iterator for EventStream {
    type Item = Event;
    fn next(&mut self) -> Option<Event> {
        self.next_event()
    }
}
