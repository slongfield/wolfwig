use cpu;

pub struct Cpu {}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {}
    }

    pub fn dump_instructions(&self, rom: &Vec<u8>, start_pc: usize, end_pc: usize) {
        let mut pc = start_pc;
        loop {
            let (op, size, _) = cpu::decode::decode(rom, pc);
            println!("0x{:x}: {} ", pc, op);
            pc += size;
            if pc >= end_pc {
                break;
            }
        }
    }
}
