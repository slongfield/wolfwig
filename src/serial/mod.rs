///! Model of the serial data peripheral.
use mem::model::Memory;
use std::sync::mpsc;

pub struct Serial {
    // The serial port has a channel connected to it that it sends data along whenever it sees a
    // serial transfer start. This is an internal detail used for testing--test roms send their
    // status information to both the serial port and to the screen, but testing serial port data
    // is simpler in automated testing.
    channel: Option<mpsc::Sender<u8>>,
}

const SERIAL_DATA: usize = 0xFF01;
const SERIAL_CONTROL: usize = 0xFF02;

const START: u8 = 1 << 7;

impl Serial {
    pub fn new(channel: Option<mpsc::Sender<u8>>) -> Serial {
        Serial { channel }
    }

    pub fn step(&mut self, mem: &mut Memory) {
        let control = mem.read(SERIAL_CONTROL);
        if (control & START) != 0 {
            if let Some(ref mut sender) = self.channel {
                let data = mem.read(SERIAL_DATA);
                // TODO(slongfield): Handle error.
                sender.send(data).unwrap();
            }
            mem.write(SERIAL_CONTROL, control & !START);
            // TODO(slongfield): Two-way communication. Normally data is shifted in here from the
            // external source as its shifted out over the course of 8 cycles.
            mem.write(SERIAL_DATA, 0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_serial_write() {
        let (tx, rx) = mpsc::channel();
        let mut mem = Memory::new(vec![0; 0x1000]);
        let mut serial = Serial::new(Some(tx));

        mem.write(SERIAL_DATA, 0x51);
        mem.write(SERIAL_CONTROL, START);

        serial.step(&mut mem);

        assert_eq!(mem.read(SERIAL_DATA), 0);
        assert_eq!(mem.read(SERIAL_CONTROL), 0);
        assert_eq!(rx.recv().unwrap(), 0x51);
    }
}
