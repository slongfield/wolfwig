///! Model of the serial data peripheral.
use std::sync::mpsc;

pub struct Serial {
    // The serial port has a channel connected to it that it sends data along whenever it sees a
    // serial transfer start. This is an internal detail used for testing--test roms send their
    // status information to both the serial port and to the screen, but testing serial port data
    // is simpler in automated testing.
    channel: Option<mpsc::Sender<u8>>,
    start: bool,
    data: u8,
}

impl Serial {
    pub fn new(channel: Option<mpsc::Sender<u8>>) -> Self {
        Self {
            channel,
            start: false,
            data: 0,
        }
    }

    pub fn step(&mut self) {
        if self.start {
            if let Some(ref mut sender) = self.channel {
                // TODO(slongfield): Handle error.
                sender.send(self.data).unwrap();
            }
            self.start = false;
            // TODO(slongfield): Two-way communication. Normally data is shifted in here from the
            // external source as its shifted out over the course of 8 cycles.
            self.data = 0;
        }
    }

    pub fn connect_channel(&mut self, tx: mpsc::Sender<u8>) {
        self.channel = Some(tx)
    }

    pub fn set_start(&mut self, val: bool) {
        self.start = val;
    }

    pub fn start(&self) -> bool {
        self.start
    }

    pub fn set_data(&mut self, val: u8) {
        self.data = val;
    }

    pub fn data(&self) -> u8 {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_serial_write() {
        let (tx, rx) = mpsc::channel();
        let mut serial = Serial::new(Some(tx));

        serial.set_data(0x51);
        serial.set_start(true);

        serial.step();

        assert_eq!(serial.data(), 0);
        assert_eq!(serial.start(), false);
        assert_eq!(rx.recv().unwrap(), 0x51);
    }
}
