pub mod decode;
pub mod header;
pub mod registers;

fn bytes_to_u32(bytes: &[u8]) -> u32 {
    let mut outp: u32 = 0;
    for byte in bytes {
        outp <<= 8;
        outp |= u32::from(*byte);
    }
    outp
}

fn bytes_to_u16(bytes: &[u8]) -> u16 {
    let mut outp: u16 = 0;
    for byte in bytes {
        outp <<= 8;
        outp |= u16::from(*byte);
    }
    outp
}

pub struct Cpu {
    regs: registers::Registers,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            regs: registers::Registers::new(),
        }
    }

    pub fn dump_instructions(&self, rom: &[u8], start_pc: usize, end_pc: usize) {
        let mut pc = start_pc;
        loop {
            let (op, size, _) = decode::decode(rom, pc);
            println!("0x{:x}: {} ", pc, op);
            pc += size;
            if pc >= end_pc {
                break;
            }
        }
    }
}
