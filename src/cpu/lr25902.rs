use self::decode::{Address, Alu16, Alu16Data, Alu16Op, Alu8, Alu8Data, Alu8Op, Op};
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
    fn new() -> Self {
        Self {
            delay_cycles: 0,
            pc_offset: 0,
            op: Op::Nop,
        }
    }
}

///! Emulation of the Sharp 8-bit LR25902 processor.
pub struct LR25902 {
    pub regs: Registers,
    next_op: NextOp,
    cycle: usize,
}

impl LR25902 {
    pub fn new() -> Self {
        Self {
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
            let pc = self.execute_op(mem, &op);
            let (op, size, cycles) = decode::decode(mem, pc as usize);
            self.next_op.op = op;
            self.next_op.pc_offset = size as u16;
            if cycles > 0 {
                self.next_op.delay_cycles = cycles - 1;
            } else {
                self.next_op.delay_cycles = 0;
            }
        } else {
            self.next_op.delay_cycles -= 1;
        }
        self.cycle += 1;
        self.regs.read16(Reg16::PC)
    }

    fn execute_op(&mut self, mem: &mut Memory, op: &NextOp) -> u16 {
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
                self.regs.set16(addr_reg, addr.wrapping_sub(1));
            }
            Op::StoreAndIncrement(Address::Register16(addr_reg), data_reg) => {
                let data = self.regs.read8(data_reg);
                let addr = self.regs.read16(addr_reg);
                mem.write(addr as usize, data);
                self.regs.set16(addr_reg, addr.wrapping_add(1));
            }
            Op::Load(reg, Address::Register16(addr_reg)) => {
                let addr = self.regs.read16(addr_reg);
                self.regs.set8(reg, mem.read(addr as usize))
            }
            Op::Load(reg, Address::Immediate16(addr)) => {
                self.regs.set8(reg, mem.read(addr as usize))
            }
            Op::LoadAndIncrement(reg, Address::Register16(addr_reg)) => {
                let addr = self.regs.read16(addr_reg);
                self.regs.set8(reg, mem.read(addr as usize));
                self.regs.set16(addr_reg, addr.wrapping_add(1));
            }
            Op::LoadAndDecrement(reg, Address::Register16(addr_reg)) => {
                let addr = self.regs.read16(addr_reg);
                self.regs.set8(reg, mem.read(addr as usize));
                self.regs.set16(addr_reg, addr.wrapping_sub(1));
            }

            Op::Call(new_pc) => {
                let sp = self.regs.read16(Reg16::SP);
                mem.write((sp - 1) as usize, ((next_pc >> 8) & 0xFF) as u8);
                mem.write((sp - 2) as usize, (next_pc & 0xFF) as u8);
                self.regs.set16(Reg16::SP, sp - 2);
                next_pc = new_pc;
            }
            Op::ConditionalCall(flag, new_pc) => {
                if self.regs.read_flag(flag) {
                    let sp = self.regs.read16(Reg16::SP);
                    mem.write((sp - 1) as usize, ((next_pc >> 8) & 0xFF) as u8);
                    mem.write((sp - 2) as usize, (next_pc & 0xFF) as u8);
                    self.regs.set16(Reg16::SP, sp - 2);
                    next_pc = new_pc;
                }
            }

            Op::Return => {
                let sp = self.regs.read16(Reg16::SP);
                let pc_low = u16::from(mem.read(sp as usize));
                let pc_high = u16::from(mem.read((sp + 1) as usize));
                self.regs.set16(Reg16::SP, sp + 2);
                next_pc = (pc_high << 8) | pc_low;
            }
            Op::ConditionalReturn(flag) => {
                if self.regs.read_flag(flag) {
                    let sp = self.regs.read16(Reg16::SP);
                    let pc_low = u16::from(mem.read(sp as usize));
                    let pc_high = u16::from(mem.read((sp + 1) as usize));
                    self.regs.set16(Reg16::SP, sp + 2);
                    next_pc = (pc_high << 8) | pc_low;
                }
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
            Op::ConditionalJump(flag, new_pc) => {
                if self.regs.read_flag(flag) {
                    next_pc = new_pc;
                }
            }
            Op::Jump(Address::Immediate16(new_pc)) => next_pc = new_pc,
            Op::Jump(Address::Register16(reg)) => {
                next_pc = self.regs.read16(reg);
            }
            Op::Alu8(ref alu_op) => self.execute_alu8(&alu_op, mem),
            Op::Alu16(ref alu_op) => self.execute_alu16(&alu_op),
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

    fn get_alu8_data(&mut self, data: &Alu8Data, mem: &mut Memory) -> u8 {
        match data {
            Alu8Data::Reg(reg) => self.regs.read8(*reg),
            Alu8Data::Imm(data) => *data,
            Alu8Data::Addr(reg16) => {
                let addr = self.regs.read16(*reg16);
                mem.read(addr as usize)
            }
            Alu8Data::Ignore => 0xFF,
        }
    }

    fn set_alu8_data(&mut self, dest: &Alu8Data, val: u8, mem: &mut Memory) {
        match dest {
            Alu8Data::Reg(reg) => self.regs.set8(*reg, val),
            Alu8Data::Addr(reg16) => {
                let addr = self.regs.read16(*reg16);
                mem.write(addr as usize, val);
            }
            other => error!("Unexpected alu8 set: {:?}", other),
        }
    }

    fn execute_alu8(&mut self, op: &Alu8Op, mem: &mut Memory) {
        let x = self.get_alu8_data(&op.dest, mem);
        let y = self.get_alu8_data(&op.y, mem);
        let (out, zero, subtract, half_carry, carry) = match op.op {
            Alu8::Add => {
                let out = (x as i8).wrapping_add(y as i8) as u8;
                let carry = u16::from(x) + u16::from(y) > 0xFF;
                // TODO(slongfield): Half Carry
                (Some(out), Some(out == 0), Some(false), None, Some(carry))
            }
            Alu8::AddWithCarry => {
                let carry_in = u8::from(self.regs.read_flag(Flag::Carry));
                let out = (x as i8).wrapping_add(y as i8).wrapping_add(carry_in as i8) as u8;
                let carry = u16::from(x) + u16::from(y) + u16::from(carry_in) > 0xFF;
                // TODO(slongfield): Half Carry
                (Some(out), Some(out == 0), Some(false), None, Some(carry))
            }
            Alu8::And => {
                let out = x & y;
                let zero = out == 0;
                (Some(out), Some(zero), Some(false), Some(true), Some(false))
            }
            Alu8::ClearCarryFlag => (None, None, None, None, Some(false)),
            Alu8::Compare => {
                let out = (x as i8).wrapping_sub(y as i8) as u8;
                let carry = i16::from(x as i8) - i16::from(y as i8) < 0;
                // TODO(slongfield): Half Carry
                (None, Some(out == 0), Some(false), None, Some(carry))
            }
            Alu8::Complement => {
                let out = !x;
                (Some(out), None, Some(true), Some(true), None)
            }
            Alu8::DecimalAdjust => (None, None, None, None, None),
            Alu8::Decrement => {
                let out = (x as i8).wrapping_sub(1) as u8;
                let half = ((x & 0xF) as i8).wrapping_sub(1) < 0;
                (Some(out), Some(out == 0), Some(true), Some(half), None)
            }
            Alu8::Increment => {
                let out = (x as i8).wrapping_add(1) as u8;
                // TODO(slongfield): Half Carry
                (Some(out), Some(out == 0), Some(false), None, None)
            }
            Alu8::Or => {
                let out = x | y;
                let zero = out == 0;
                (Some(out), Some(zero), Some(false), Some(false), Some(false))
            }
            Alu8::ResetBit => {
                let out = x & !(1 << y);
                (Some(out), None, None, None, None)
            }
            Alu8::RotateLeft => {
                let rot_data = u16::from(x) << 1;
                let carry = ((1 << 8) & rot_data) != 0;
                let out = (rot_data & 0xFF) as u8;
                let zero = out == 0;
                (Some(out), Some(zero), Some(false), Some(false), Some(carry))
            }
            Alu8::RotateLeftCarry => {
                let carry_in = u16::from(self.regs.read_flag(Flag::Carry));
                let rot_data = (u16::from(x) | (carry_in << 8)) << 1;
                let carry = ((1 << 8) & rot_data) != 0;
                let low_bit = u8::from(((1 << 9) & rot_data) != 0);
                let out = (((rot_data & 0xFF) as u8) | low_bit) as u8;
                let zero = out == 0;
                (Some(out), Some(zero), Some(false), Some(false), Some(carry))
            }
            Alu8::RotateRight => {
                let rot_data = u16::from(x) >> 1;
                let carry = u16::from(x & 1);
                let out = ((rot_data | (carry << 8)) & 0xFF) as u8;
                let zero = out == 0;
                let carry = carry != 0;
                (Some(out), Some(zero), Some(false), Some(false), Some(carry))
            }
            Alu8::RotateRightCarry => (None, None, None, None, None),
            Alu8::SetBit => {
                let out = x | (1 << y);
                (Some(out), None, None, None, None)
            }
            Alu8::SetCarryFlag => (None, None, None, None, Some(true)),
            Alu8::ShiftLeftArithmetic => (None, None, None, None, None),
            Alu8::ShiftRightArithmetic => (None, None, None, None, None),
            Alu8::ShiftRightLogical => (None, None, None, None, None),
            Alu8::Sub => {
                let out = (x as i8).wrapping_sub(y as i8) as u8;
                let carry = i16::from(x as i8) - i16::from(y as i8) < 0;
                let zero = out == 0;
                // TODO(slongfield): Half Carry
                (Some(out), Some(zero), Some(false), None, Some(carry))
            }
            Alu8::SubWithCarry => {
                let carry_in = u8::from(self.regs.read_flag(Flag::Carry));
                let out = (x as i8).wrapping_sub(y as i8).wrapping_sub(carry_in as i8) as u8;
                let carry = i16::from(x as i8) - i16::from(y as i8) - i16::from(carry_in) < 0;
                // TODO(slongfield): Half Carry
                let zero = out == 0;
                (Some(out), Some(zero), Some(false), None, Some(carry))
            }
            Alu8::Swap => (None, None, None, None, None),
            Alu8::TestBit => {
                let zero = x & (1 << y) == 0;
                (None, Some(zero), Some(false), Some(true), None)
            }
            Alu8::Xor => {
                let out = x ^ y;
                let zero = out == 0;
                (Some(out), Some(zero), Some(false), Some(false), Some(false))
            }
            Alu8::Unknown => {
                error!("Attempted to execute Unknown ALU8Op!");
                (None, None, None, None, None)
            }
        };
        if let Some(data) = out {
            self.set_alu8_data(&op.dest, data, mem);
        }
        if let Some(zero) = zero {
            self.regs.set_flag(Flag::Zero, zero);
        }
        if let Some(subtract) = subtract {
            self.regs.set_flag(Flag::Subtract, subtract);
        }
        if let Some(half_carry) = half_carry {
            self.regs.set_flag(Flag::HalfCarry, half_carry);
        }
        if let Some(carry) = carry {
            self.regs.set_flag(Flag::Carry, carry);
        }
    }

    fn execute_alu16(&mut self, op: &Alu16Op) {
        match op.op {
            Alu16::Add => match op.y {
                Alu16Data::Reg(yreg) => {
                    let x = self.regs.read16(op.dest);
                    let y = self.regs.read16(yreg);
                    let out = x.wrapping_add(y);
                    self.regs.set16(op.dest, out);
                }
                Alu16Data::Imm(data) => {
                    let x = self.regs.read16(op.dest);
                    let out = x.wrapping_add(data.into());
                    self.regs.set16(op.dest, out);
                }
                Alu16Data::Ignore => {}
            },
            Alu16::Decrement => {
                let x = self.regs.read16(op.dest);
                self.regs.set16(op.dest, x.wrapping_sub(1));
            }
            Alu16::Increment => {
                let x = self.regs.read16(op.dest);
                self.regs.set16(op.dest, x.wrapping_add(1));
            }
            Alu16::Unknown => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotate_left_carry() {
        let mut cpu = LR25902::new();
        cpu.regs.set8(Reg8::A, 0xFF);

        let mut mem = Memory::new(vec![0; 0x100], vec![0; 0x1000]);

        let op = Alu8Op {
            op: Alu8::RotateLeftCarry,
            dest: Alu8Data::Reg(Reg8::A),
            y: Alu8Data::Ignore,
        };

        cpu.execute_alu8(&op, &mut mem);

        assert_eq!(cpu.regs.read8(Reg8::A), (0xFF << 1) & 0xFF);
        assert_eq!(cpu.regs.read_flag(Flag::Carry), true);

        cpu.execute_alu8(&op, &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 0b1111_1101);
    }
}
