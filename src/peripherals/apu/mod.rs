///! Model of the Audio Processing Unit

pub struct Apu {}

impl Apu {
    pub fn new() -> Self {
        Self {}
    }

    pub fn step(&self) {}

    pub fn write(&mut self, addr: u16, _val: u8) {
        match addr {
            0xFF10..=0xFF3F => info!("Writing APU I/O reg: {:#04X}", addr),
            _ => panic!("Attempted to write APU with unmapped addr: {:#x}", addr),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF10..=0xFF3F => info!("Reading APU I/O reg: {:#04X}", addr),
            _ => panic!("Attempted to read APU with unmapped addr: {:#x}", addr),
        };
        0
    }
}
