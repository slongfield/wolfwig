///! Interface that needs to be implemented to create a `Joypad`

#[derive(Copy, Clone, Debug)]
pub struct State {
    pub shutdown: bool,
    pub start: bool,
    pub select: bool,
    pub a: bool,
    pub b: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    // This is set true if a button is pressed. Should be cleared by the joypad controller when
    // read.
    pub keydown: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            shutdown: false,
            start: false,
            select: false,
            a: false,
            b: false,
            up: false,
            down: false,
            left: false,
            right: false,
            keydown: false,
        }
    }
}

pub trait EventHandler {
    fn get_state(&mut self) -> State;
    fn clear_keydown(&mut self);
}
