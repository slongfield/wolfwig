///! Model of the serial data peripheral.
use std::sync::mpsc;

pub struct Serial {
    // The serial port has a channel connected to it that it sends data along whenever it sees a
    // serial transfer start. This is an internal detail used for testing--test roms send their
    // status information to both the serial port and to the screen, but testing serial port data
    // is simpler in automated testing.
    channel: Option<mpsc::Sender<u8>>,
    data: u8,
    control: u8,
}

impl Serial {
    const DATA: u16 = 0xFF01;
    const CONTROL: u16 = 0xFF02;
    const START: u8 = 1 << 7;

    pub fn new(channel: Option<mpsc::Sender<u8>>) -> Self {
        Self {
            channel,
            control: 0,
            data: 0,
        }
    }

    pub fn step(&mut self) {
        if (self.control & Self::START) != 0 {
            if let Some(ref mut sender) = self.channel {
                // TODO(slongfield): Handle error.
                sender.send(self.data).unwrap();
            }
            self.control = self.control & !Self::START;
            // TODO(slongfield): Two-way communication. Normally data is shifted in here from the
            // external source as its shifted out over the course of 8 cycles.
            self.data = 0;
        }
    }

    pub fn connect_channel(&mut self, tx: mpsc::Sender<u8>) {
        self.channel = Some(tx)
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            Self::DATA => self.data = val,
            Self::CONTROL => self.control = val,
            _ => panic!("Attempted to write Serial with unmapped addr: {:#x}", addr),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            Self::DATA => self.data,
            Self::CONTROL => self.control,
            _ => panic!("Attempted to read serial with unmapped addr: {:#x}", addr),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_serial_write() {
        let (tx, rx) = mpsc::channel();
        let mut serial = Serial::new(Some(tx));

        serial.write(Serial::DATA, 0x51);
        serial.write(Serial::CONTROL, Serial::START);

        serial.step();

        assert_eq!(serial.read(Serial::DATA), 0);
        assert_eq!(serial.read(Serial::CONTROL), 0);
        assert_eq!(rx.recv().unwrap(), 0x51);
    }
}
