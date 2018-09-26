use self::decode::{Address, AluOp, Data, Op};
use cpu::decode;
use cpu::registers::{Flag, Reg16, Reg8, Registers};
use mem::model::Memory;
use std::mem;

struct NextOp {
    delay_cycles: usize,
    pc_offset: u16,
    op: Op,
}

impl NextOp {
    fn new() -> NextOp {
        NextOp {
            delay_cycles: 0,
            pc_offset: 0,
            op: Op::Nop,
        }
    }
}

///! Emulation of the Sharp 8-bit LR25902 processor.
pub struct LR25902 {
    regs: Registers,
    next_op: NextOp,
    cycle: usize,
}

impl LR25902 {
    pub fn new() -> LR25902 {
        LR25902 {
            regs: Registers::new(),
            next_op: NextOp::new(),
            cycle: 0,
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

    pub fn step(&mut self, mem: &mut Memory) -> u16 {
        // TODO(slongfield): Handle interrupts.
        info!(
            "Executing cycle: {}, pc: {}",
            self.cycle,
            self.regs.read16(Reg16::PC)
        );
        if self.next_op.delay_cycles == 0 {
            let op = mem::replace(&mut self.next_op, NextOp::new());
            let pc = self.execute_op(mem, op);
            let (op, size, cycles) = decode::decode(mem, pc as usize);
            self.next_op.op = op;
            self.next_op.pc_offset = size as u16;
            self.next_op.delay_cycles = cycles - 1;
        } else {
            self.next_op.delay_cycles -= 1;
        }
        self.cycle += 1;
        self.regs.read16(Reg16::PC)
    }

    fn execute_op(&mut self, mem: &mut Memory, op: NextOp) -> u16 {
        //println!("Executing op {} ", op.op);
        let pc = self.regs.read16(Reg16::PC);
        let mut next_pc = pc + op.pc_offset;
        match op.op {
            Op::Nop => {}
            Op::Set(reg, val) => self.regs.set8(reg, val),
            Op::SetWide(reg, val) => self.regs.set16(reg, val),
            Op::SetIOC => {
                let a = self.regs.read8(Reg8::A);
                let c = self.regs.read8(Reg8::C);
                mem.write(0xFF00 + c as usize, a);
            }
            Op::SetIO(addr) => {
                let a = self.regs.read8(Reg8::A);
                mem.write(0xFF00 + addr as usize, a);
            }
            Op::ReadIO(addr) => {
                let data = mem.read(0xFF00 + addr as usize);
                self.regs.set8(Reg8::A, data)
            }
            Op::ReadIOC => {
                let addr = self.regs.read8(Reg8::C);
                let data = mem.read(0xFF00 + addr as usize);
                self.regs.set8(Reg8::A, data)
            }
            Op::Store(Address::Register16(addr_reg), data_reg) => {
                let data = self.regs.read8(data_reg);
                let addr = self.regs.read16(addr_reg);
                mem.write(addr as usize, data);
            }
            Op::Store(Address::Immediate16(addr), data_reg) => {
                let data = self.regs.read8(data_reg);
                mem.write(addr as usize, data);
            }
            Op::StoreAndDecrement(Address::Register16(addr_reg), data_reg) => {
                let data = self.regs.read8(data_reg);
                let addr = self.regs.read16(addr_reg);
                mem.write(addr as usize, data);
                self.regs.set16(addr_reg, addr - 1);
            }
            Op::StoreAndIncrement(Address::Register16(addr_reg), data_reg) => {
                let data = self.regs.read8(data_reg);
                let addr = self.regs.read16(addr_reg);
                mem.write(addr as usize, data);
                self.regs.set16(addr_reg, addr + 1);
            }
            Op::Load(reg, Address::Register16(addr_reg)) => {
                let addr = self.regs.read16(addr_reg);
                self.regs.set8(reg, mem.read(addr as usize))
            }
            Op::Call(new_pc) => {
                let sp = self.regs.read16(Reg16::SP);
                mem.write((sp - 1) as usize, ((next_pc >> 8) & 0xFF) as u8);
                mem.write((sp - 2) as usize, (next_pc & 0xFF) as u8);
                self.regs.set16(Reg16::SP, sp - 2);
                next_pc = new_pc;
            }
            Op::Return => {
                let sp = self.regs.read16(Reg16::SP);
                let pc_low = u16::from(mem.read(sp as usize));
                let pc_high = u16::from(mem.read((sp + 1) as usize));
                self.regs.set16(Reg16::SP, sp + 2);
                next_pc = (pc_high << 8) | pc_low;
            }
            Op::Move(dest, src) => {
                let data = self.regs.read8(src);
                self.regs.set8(dest, data);
            }
            Op::Push(reg) => {
                // TODO(slongfield): Commonize this code with Call
                let data = self.regs.read16(reg);
                let sp = self.regs.read16(Reg16::SP);
                mem.write((sp - 1) as usize, ((data >> 8) & 0xFF) as u8);
                mem.write((sp - 2) as usize, (data & 0xFF) as u8);
                self.regs.set16(Reg16::SP, sp - 2);
            }
            Op::Pop(reg) => {
                // TODO(slongfield): Commonize this code with Return
                let sp = self.regs.read16(Reg16::SP);
                let data_low = u16::from(mem.read(sp as usize));
                let data_high = u16::from(mem.read((sp + 1) as usize));
                self.regs.set16(Reg16::SP, sp + 2);
                self.regs.set16(reg, (data_high << 8) | data_low);
            }
            Op::ConditionalJumpRelative(flag, new_pc) => {
                // TODO(slongfield): When this branch is taken, it should consume an additional
                // cycle.
                if self.regs.read_flag(flag) {
                    next_pc = new_pc
                }
            }
            Op::JumpRelative(new_pc) => next_pc = new_pc,
            Op::AluOp(ref alu_op) => self.execute_alu_op(&alu_op, mem),
            _ => error!(
                "Cycle: {} PC: 0x{:04X} Unknown op: {:?}",
                self.cycle,
                self.regs.read16(Reg16::PC),
                op.op
            ),
        }
        //println!("{}", self.regs);
        self.regs.set16(Reg16::PC, next_pc);
        next_pc
    }

    fn execute_alu_op(&mut self, op: &AluOp, mem: &mut Memory) {
        match op {
            AluOp::Xor(Data::Immediate8(data)) => {
                let a = self.regs.read8(Reg8::A);
                self.regs.set8(Reg8::A, a ^ data);
            }
            AluOp::Xor(Data::Register8(reg)) => {
                let a = self.regs.read8(Reg8::A);
                let data = self.regs.read8(*reg);
                self.regs.set8(Reg8::A, a ^ data);
            }
            AluOp::TestBit(reg, bit) => {
                let data = self.regs.read8(*reg);
                self.regs
                    .set_flag(Flag::Zero, (data & (u8::from(1) << bit)) == 0);
            }
            AluOp::Dec(reg) => {
                let data = self.regs.read8(*reg);
                self.regs.set8(*reg, data - 1);
            }
            AluOp::WideDec(reg) => {
                let data = self.regs.read16(*reg);
                self.regs.set16(*reg, data - 1);
            }
            AluOp::AddrDec(addr_reg) => {
                let addr = self.regs.read16(*addr_reg) as usize;
                let data = mem.read(addr);
                mem.write(addr, data - 1);
            }
            AluOp::Inc(reg) => {
                let data = self.regs.read8(*reg);
                self.regs.set8(*reg, data + 1);
            }
            AluOp::WideInc(reg) => {
                let data = self.regs.read16(*reg);
                self.regs.set16(*reg, data + 1);
            }

            AluOp::AddrInc(addr_reg) => {
                let addr = self.regs.read16(*addr_reg) as usize;
                let data = mem.read(addr);
                mem.write(addr, data + 1);
            }

            _ => error!(
                "Cycle: {} PC: 0x{:04X} Unknown ALU op: {:?}. Regs: {}",
                self.cycle,
                self.regs.read16(Reg16::PC),
                op,
                self.regs
            ),
        }
    }
}
