use cpu::decode;
use cpu::registers::Registers;
use mem::model::Memory;

///! Emulation of the Sharp 8-bit LR25902 processor.
pub struct LR25902 {
    regs: Registers,
}

impl LR25902 {
    pub fn new() -> LR25902 {
        LR25902 {
            regs: Registers::new(),
        }
    }

    pub fn dump_instructions(&self, rom: &Memory, start_pc: usize, end_pc: usize) {
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
