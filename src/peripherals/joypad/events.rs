///! Interface that needs to be implemented to create a JoyPad

pub enum Event {
    PowerOff,
    Start,
    Select,
    A,
    B,
    Up,
    Down,
    Left,
    Right,
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
