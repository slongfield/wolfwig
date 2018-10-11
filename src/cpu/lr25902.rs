use self::decode::{Address, Alu16, Alu16Data, Alu16Op, Alu8, Alu8Data, Alu8Op, Op};
use cpu::decode;
use cpu::registers::{Flag, Reg16, Reg8, Registers};
use mem::model::Memory;
use std::mem;
use std::process;

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
    interrupt_enable: bool,
    halted: bool,
    stopped: bool,
}

impl LR25902 {
    pub fn new() -> Self {
        Self {
            regs: Registers::new(),
            next_op: NextOp::new(),
            cycle: 0,
            interrupt_enable: false,
            halted: false,
            stopped: false,
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

    pub fn step(&mut self, mem: &mut Memory) -> bool {
        // TODO(slongfield): Handle interrupts.
        info!(
            "Executing cycle: {}, pc: {}",
            self.cycle,
            self.regs.read16(Reg16::PC)
        );
        if self.next_op.delay_cycles == 0 && !self.halted {
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
        self.stopped
    }

    pub fn pc(&self) -> u16 {
        self.regs.read16(Reg16::PC)
    }

    fn execute_op(&mut self, mem: &mut Memory, op: &NextOp) -> u16 {
        let pc = self.regs.read16(Reg16::PC);
        let mut next_pc = pc + op.pc_offset;
        match op.op {
            Op::Nop => {}
            Op::EnableInterrupts => {
                self.interrupt_enable = true;
            }
            Op::DisableInterrupts => {
                self.interrupt_enable = false;
            }
            Op::Halt => {
                // TODO(slongfield): Add halted bug. If interrupts are not enabled. Halt skips the
                // next instruction.
                self.halted = true;
            }
            Op::Stop => {
                // TODO(slongfield): Should only stop until a button is pressed.
                self.stopped = true
            }

            Op::Set(reg, val) => self.regs.set8(reg, val),
            Op::SetWide(reg, val) => self.regs.set16(reg, val),
            Op::SetAddr(reg, val) => {
                let addr = self.regs.read16(reg);
                mem.write(addr as usize, val);
            }
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
            Op::WideStore(Address::Register16(addr_reg), data_reg) => {
                let data = self.regs.read16(data_reg);
                let addr = self.regs.read16(addr_reg);
                mem.write(addr as usize, data as u8);
                mem.write((addr + 1) as usize, (data >> 8) as u8);
            }
            Op::WideStore(Address::Immediate16(addr), data_reg) => {
                let data = self.regs.read16(data_reg);
                mem.write(addr as usize, data as u8);
                mem.write((addr + 1) as usize, (data >> 8) as u8);
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
            Op::ReturnAndEnableInterrupts => {
                let sp = self.regs.read16(Reg16::SP);
                let pc_low = u16::from(mem.read(sp as usize));
                let pc_high = u16::from(mem.read((sp + 1) as usize));
                self.regs.set16(Reg16::SP, sp + 2);
                self.interrupt_enable = true;
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
                let data = self.regs.read16(reg);
                let sp = self.regs.read16(Reg16::SP);
                mem.write((sp - 1) as usize, ((data >> 8) & 0xFF) as u8);
                mem.write((sp - 2) as usize, (data & 0xFF) as u8);
                self.regs.set16(Reg16::SP, sp - 2);
            }
            Op::Pop(reg) => {
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

            // This is basically the same as call.
            Op::Reset(new_pc) => {
                let sp = self.regs.read16(Reg16::SP);
                mem.write((sp - 1) as usize, ((next_pc >> 8) & 0xFF) as u8);
                mem.write((sp - 2) as usize, (next_pc & 0xFF) as u8);
                self.regs.set16(Reg16::SP, sp - 2);
                next_pc = new_pc;
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
                let h = (x & 0xF) + (y & 0xF) > 0xF;
                (Some(out), Some(out == 0), Some(false), Some(h), Some(carry))
            }
            Alu8::AddWithCarry => {
                let carry_in = u8::from(self.regs.read_flag(Flag::Carry));
                let out = (x as i8).wrapping_add(y as i8).wrapping_add(carry_in as i8) as u8;
                let carry = u16::from(x) + u16::from(y) + u16::from(carry_in) > 0xFF;
                let h = (x & 0xF) + (y & 0xF) + carry_in > 0xF;
                (Some(out), Some(out == 0), Some(false), Some(h), Some(carry))
            }
            Alu8::And => {
                let out = x & y;
                let zero = out == 0;
                (Some(out), Some(zero), Some(false), Some(true), Some(false))
            }
            Alu8::ClearCarryFlag => (None, None, None, None, Some(false)),
            Alu8::Compare => {
                let out = (x as i8).wrapping_sub(y as i8) as u8;
                let compy = u16::from(!y) + 1;
                let carry = compy.wrapping_add(u16::from(x)) <= 0xFF;
                let h = i16::from((x & 0xF) as i8).wrapping_sub(i16::from((y & 0xF) as i8)) < 0;
                (None, Some(out == 0), Some(true), Some(h), Some(carry))
            }
            Alu8::Complement => {
                let out = !x;
                (Some(out), None, Some(true), Some(true), None)
            }
            Alu8::DecimalAdjust => {
                let subtract = self.regs.read_flag(Flag::Subtract);
                let carry_in = self.regs.read_flag(Flag::Carry);
                let half_carry_in = self.regs.read_flag(Flag::HalfCarry);
                let mut out = u16::from(x);
                let mut carry = None;

                if subtract {
                    if half_carry_in {
                        out = out.wrapping_sub(6);
                    }
                    if carry_in {
                        out = out.wrapping_sub(0x60);
                    }
                } else {
                    let low_nibble = out & 0xF;
                    if low_nibble > 9 || half_carry_in {
                        out = out.wrapping_add(6);
                    }
                    let high_nibble = out >> 4;
                    if high_nibble > 9 || carry_in {
                        out = out.wrapping_add(0x60);
                        carry = Some(true);
                    }
                }

                let out = (out & 0xFF) as u8;
                let zero = out == 0;
                (Some(out), Some(zero), None, Some(false), carry)
            }
            Alu8::Decrement => {
                let out = (x as i8).wrapping_sub(1) as u8;
                let half = ((x & 0xF) as i8).wrapping_sub(1) < 0;
                (Some(out), Some(out == 0), Some(true), Some(half), None)
            }
            Alu8::Increment => {
                let out = (x as i8).wrapping_add(1) as u8;
                let half = ((x & 0xF) as i8).wrapping_add(1) > 0xF;
                (Some(out), Some(out == 0), Some(false), Some(half), None)
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
                let carry_in = u16::from(self.regs.read_flag(Flag::Carry));
                let rot_data = (u16::from(x) | (carry_in << 8)) << 1;
                let carry = ((1 << 8) & rot_data) != 0;
                let low_bit = u8::from(((1 << 9) & rot_data) != 0);
                let out = (((rot_data & 0xFF) as u8) | low_bit) as u8;
                let zero = out == 0;
                (Some(out), Some(zero), Some(false), Some(false), Some(carry))
            }
            Alu8::RotateLeftCarry => {
                let rot_data = u16::from(x) << 1;
                let carry = ((1 << 8) & rot_data) != 0;
                let out = (rot_data & 0xFF) as u8;
                let zero = out == 0;
                (Some(out), Some(zero), Some(false), Some(false), Some(carry))
            }
            Alu8::RotateRight => {
                let carry_in = u16::from(self.regs.read_flag(Flag::Carry));
                let rot_data = u16::from(x >> 1);
                let carry = u16::from(x & 1);
                let out = ((rot_data | (carry_in << 7)) & 0xFF) as u8;
                let zero = out == 0;
                let carry = carry != 0;
                (Some(out), Some(zero), Some(false), Some(false), Some(carry))
            }
            Alu8::RotateRightCarry => {
                let rot_data = u16::from(x) >> 1;
                let carry = u16::from(x & 1);
                let out = ((rot_data | (carry << 7)) & 0xFF) as u8;
                let zero = out == 0;
                let carry = carry != 0;
                (Some(out), Some(zero), Some(false), Some(false), Some(carry))
            }
            Alu8::SetBit => {
                let out = x | (1 << y);
                (Some(out), None, None, None, None)
            }
            Alu8::SetCarryFlag => (None, None, None, None, Some(true)),
            Alu8::ShiftLeftArithmetic => {
                let carry = (x & (1 << 7)) != 0;
                let out = (u16::from(x) << 1) as u8;
                let zero = out == 0;
                (Some(out), Some(zero), Some(false), Some(false), Some(carry))
            }
            Alu8::ShiftRightArithmetic => {
                let top_bit = u16::from(x & (1 << 7));
                let carry = (x & 1) != 0;
                let out = ((u16::from(x) >> 1) | top_bit) as u8;
                let zero = out == 0;
                (Some(out), Some(zero), Some(false), Some(false), Some(carry))
            }
            Alu8::ShiftRightLogical => {
                let out = x >> 1;
                let carry = u16::from(x & 1);
                let zero = out == 0;
                let carry = carry != 0;
                (Some(out), Some(zero), Some(false), Some(false), Some(carry))
            }
            Alu8::Sub => {
                let out = (x as i8).wrapping_sub(y as i8) as u8;
                let compy = u16::from(!y) + 1;
                let carry = compy.wrapping_add(u16::from(x)) <= 0xFF;
                let h = i16::from((x & 0xF) as i8).wrapping_sub(i16::from((y & 0xF) as i8)) < 0;
                let zero = out == 0;
                (Some(out), Some(zero), Some(true), Some(h), Some(carry))
            }
            Alu8::SubWithCarry => {
                let carry_in = u8::from(self.regs.read_flag(Flag::Carry));
                let out = (x as i8).wrapping_sub(y as i8).wrapping_sub(carry_in as i8) as u8;
                let h = i16::from((x & 0xF) as i8)
                    .wrapping_sub(i16::from((y & 0xF) as i8))
                    .wrapping_sub(i16::from(carry_in))
                    < 0;
                let compy = u16::from(!y) + 1;
                let carry = compy
                    .wrapping_add(u16::from(x))
                    .wrapping_sub(u16::from(carry_in))
                    <= 0xFF;
                let zero = out == 0;
                (Some(out), Some(zero), Some(true), Some(h), Some(carry))
            }
            Alu8::Swap => {
                let low_nibble = u16::from(x & 0xF);
                let high_nibble = u16::from(x >> 4);
                let out = ((low_nibble << 4) | high_nibble) as u8;
                let zero = out == 0;
                (Some(out), Some(zero), None, None, None)
            }
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
        let (zero, subtract, half_carry, carry) = match op.op {
            Alu16::Add => match op.y {
                Alu16Data::Reg(yreg) => {
                    let x = self.regs.read16(op.dest) as i16;
                    let y = self.regs.read16(yreg) as i16;
                    let out = x.wrapping_add(y);
                    let carry = ((x as u16) as u32) + ((y as u16) as u32) > 0xFFFF;
                    let half = ((x & 0xFFF) as u32) + ((y & 0xFFF) as u32) > 0xFFF;
                    self.regs.set16(op.dest, out as u16);
                    (None, Some(false), Some(half), Some(carry))
                }
                Alu16Data::Imm(data) => {
                    let x = self.regs.read16(op.dest) as i16;
                    let out = x.wrapping_add(data.into());
                    let carry = ((x & 0xFF) as u16) + ((data as u8) as u16) > 0xFF;
                    let half = ((x & 0xF) as u16) + (((data as u8) & 0xF) as u16) > 0xF;
                    self.regs.set16(op.dest, out as u16);
                    (Some(false), Some(false), Some(half), Some(carry))
                }
                Alu16Data::Ignore => (None, None, None, None),
            },
            Alu16::Decrement => {
                let x = self.regs.read16(op.dest);
                self.regs.set16(op.dest, x.wrapping_sub(1));
                (None, None, None, None)
            }
            Alu16::Increment => {
                let x = self.regs.read16(op.dest);
                self.regs.set16(op.dest, x.wrapping_add(1));
                (None, None, None, None)
            }
            Alu16::Move => {
                if let Alu16Data::Reg(yreg) = op.y {
                    let y = self.regs.read16(yreg);
                    self.regs.set16(op.dest, y);
                }
                (None, None, None, None)
            }
            Alu16::MoveAndAdd => {
                if let Alu16Data::Reg(yreg) = op.y {
                    let y = self.regs.read16(yreg) as i16;
                    let imm = op.imm;
                    self.regs.set16(op.dest, y.wrapping_add(imm as i16) as u16);
                    let carry = ((y & 0xFF) as u16) + ((op.imm as u8) as u16) > 0xFF;
                    let half = ((y & 0xF) as u16) + (((op.imm as u8) & 0xF) as u16) > 0xF;

                    (Some(false), Some(false), Some(half), Some(carry))
                } else {
                    error!("Invalid MoveAndAdd");
                    (None, None, None, None)
                }
            }
            Alu16::Unknown => {
                error!("Executing unknown ALU 16 Op!");
                (None, None, None, None)
            }
        };
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotate_left_carry() {
        let mut cpu = LR25902::new();
        let mut mem = Memory::new(vec![0; 0x100], vec![0; 0x1000]);

        cpu.regs.set8(Reg8::A, 0xFF);

        let op = Alu8Op {
            op: Alu8::RotateLeft,
            dest: Alu8Data::Reg(Reg8::A),
            y: Alu8Data::Ignore,
        };

        cpu.execute_alu8(&op, &mut mem);

        assert_eq!(cpu.regs.read8(Reg8::A), (0xFF << 1) & 0xFF);
        assert_eq!(cpu.regs.read_flag(Flag::Carry), true);

        cpu.execute_alu8(&op, &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 0b1111_1101);
    }

    #[test]
    fn rotate_right() {
        let mut cpu = LR25902::new();
        let mut mem = Memory::new(vec![0; 0x100], vec![0; 0x1000]);

        cpu.regs.set8(Reg8::A, 0xFF);
        cpu.regs.set_flag(Flag::Carry, true);

        let op = Alu8Op {
            op: Alu8::RotateRight,
            dest: Alu8Data::Reg(Reg8::A),
            y: Alu8Data::Ignore,
        };

        cpu.execute_alu8(&op, &mut mem);

        assert_eq!(cpu.regs.read8(Reg8::A), 0xFF);
        assert_eq!(cpu.regs.read_flag(Flag::Carry), true);
    }

    #[test]
    fn decrement_half_carry_test() {
        let mut cpu = LR25902::new();
        let mut mem = Memory::new(vec![0; 0x100], vec![0; 0x1000]);

        cpu.regs.set8(Reg8::A, 0);

        let op = Alu8Op {
            op: Alu8::Decrement,
            dest: Alu8Data::Reg(Reg8::A),
            y: Alu8Data::Ignore,
        };

        cpu.execute_alu8(&op, &mut mem);

        assert_eq!(cpu.regs.read8(Reg8::A), 0xFF);
        assert_eq!(cpu.regs.read_flag(Flag::Zero), false);
        assert_eq!(cpu.regs.read_flag(Flag::Subtract), true);
        assert_eq!(cpu.regs.read_flag(Flag::HalfCarry), true);
        assert_eq!(cpu.regs.read_flag(Flag::Carry), false);
    }

    #[test]
    fn sub() {
        let mut cpu = LR25902::new();
        let mut mem = Memory::new(vec![0; 0x100], vec![0; 0x1000]);

        let make_sub = |val| Alu8Op {
            op: Alu8::Sub,
            dest: Alu8Data::Reg(Reg8::A),
            y: Alu8Data::Imm(val),
        };

        // 0 - 0 == 0
        cpu.regs.set8(Reg8::A, 0);
        cpu.execute_alu8(&make_sub(0), &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 0);
        assert_eq!(cpu.regs.read_flag(Flag::Zero), true);
        assert_eq!(cpu.regs.read_flag(Flag::Subtract), true);
        assert_eq!(cpu.regs.read_flag(Flag::HalfCarry), false);
        assert_eq!(cpu.regs.read_flag(Flag::Carry), false);

        // 0 - 0x0F == 0xF1
        cpu.regs.set8(Reg8::A, 0);
        cpu.execute_alu8(&make_sub(0x0F), &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 0xF1);
        assert_eq!(cpu.regs.read_flag(Flag::Zero), false);
        assert_eq!(cpu.regs.read_flag(Flag::Subtract), true);
        assert_eq!(cpu.regs.read_flag(Flag::HalfCarry), true);
        assert_eq!(cpu.regs.read_flag(Flag::Carry), true);

        // 0 - 0xF0 == 0x10
        cpu.regs.set8(Reg8::A, 0);
        cpu.execute_alu8(&make_sub(0xF0), &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 0x10);
        assert_eq!(cpu.regs.read_flag(Flag::Zero), false);
        assert_eq!(cpu.regs.read_flag(Flag::Subtract), true);
        assert_eq!(cpu.regs.read_flag(Flag::HalfCarry), false);
        assert_eq!(cpu.regs.read_flag(Flag::Carry), true);

        // 0xFF - 0xFF == 0
        cpu.regs.set8(Reg8::A, 0xFF);
        cpu.execute_alu8(&make_sub(0xFF), &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 0);
        assert_eq!(cpu.regs.read_flag(Flag::Zero), true);
        assert_eq!(cpu.regs.read_flag(Flag::Subtract), true);
        assert_eq!(cpu.regs.read_flag(Flag::HalfCarry), false);
        assert_eq!(cpu.regs.read_flag(Flag::Carry), false);
    }

    #[test]
    fn sbc() {
        let mut cpu = LR25902::new();
        let mut mem = Memory::new(vec![0; 0x100], vec![0; 0x1000]);

        let make_sbc = |val| Alu8Op {
            op: Alu8::SubWithCarry,
            dest: Alu8Data::Reg(Reg8::A),
            y: Alu8Data::Imm(val),
        };

        // 13 - 13, C = 0xFF, H, C
        cpu.regs.set8(Reg8::A, 17);
        cpu.regs.set_flag(Flag::Carry, true);
        cpu.execute_alu8(&make_sbc(17), &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 0xFF);
        assert_eq!(cpu.regs.read_flag(Flag::Zero), false);
        assert_eq!(cpu.regs.read_flag(Flag::Subtract), true);
        assert_eq!(cpu.regs.read_flag(Flag::HalfCarry), true);
        assert_eq!(cpu.regs.read_flag(Flag::Carry), true);

        // 5 - 2, C = 2
        cpu.regs.set8(Reg8::A, 5);
        cpu.regs.set_flag(Flag::Carry, true);
        cpu.execute_alu8(&make_sbc(2), &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 2);
        assert_eq!(cpu.regs.read_flag(Flag::Zero), false);
        assert_eq!(cpu.regs.read_flag(Flag::Subtract), true);
        assert_eq!(cpu.regs.read_flag(Flag::HalfCarry), false);
        assert_eq!(cpu.regs.read_flag(Flag::Carry), false);

        // 7F - 80, C = 0xFE C
        cpu.regs.set8(Reg8::A, 0x7F);
        cpu.regs.set_flag(Flag::Carry, true);
        cpu.execute_alu8(&make_sbc(0x80), &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 0xFE);
        assert_eq!(cpu.regs.read_flag(Flag::Zero), false);
        assert_eq!(cpu.regs.read_flag(Flag::Subtract), true);
        assert_eq!(cpu.regs.read_flag(Flag::HalfCarry), false);
        assert_eq!(cpu.regs.read_flag(Flag::Carry), true);
    }

    #[test]
    fn alu16_add() {
        let mut cpu = LR25902::new();

        cpu.regs.set16(Reg16::HL, 0x0F00);
        cpu.regs.set16(Reg16::SP, 0x8000);
        cpu.execute_alu16(&Alu16Op {
            op: Alu16::Add,
            dest: Reg16::HL,
            y: Alu16Data::Reg(Reg16::SP),
            imm: 0,
        });

        assert_eq!(cpu.regs.read16(Reg16::HL), 0x8F00);
        assert_eq!(cpu.regs.read_flag(Flag::Zero), false);
        assert_eq!(cpu.regs.read_flag(Flag::Subtract), false);
        assert_eq!(cpu.regs.read_flag(Flag::HalfCarry), false);
        assert_eq!(cpu.regs.read_flag(Flag::Carry), false);
    }

    #[test]
    fn alu16_move_and_add() {
        let mut cpu = LR25902::new();

        cpu.regs.set16(Reg16::HL, 0x4321);
        cpu.regs.set16(Reg16::SP, 0x1234);
        cpu.execute_alu16(&Alu16Op {
            op: Alu16::MoveAndAdd,
            dest: Reg16::HL,
            y: Alu16Data::Reg(Reg16::SP),
            imm: -1,
        });

        assert_eq!(cpu.regs.read16(Reg16::HL), 0x1233);
    }

    #[test]
    fn push_and_pop() {
        let mut cpu = LR25902::new();
        let mut mem = Memory::new(vec![0; 0x100], vec![0; 0x1000]);

        cpu.regs.set16(Reg16::AF, 0x12FF);
        cpu.regs.set16(Reg16::BC, 0x13FF);
        cpu.regs.set16(Reg16::DE, 0x14FF);
        cpu.regs.set16(Reg16::HL, 0x15FF);
        cpu.regs.set16(Reg16::SP, 0xFFFF);

        let make_push = |reg| NextOp {
            delay_cycles: 0,
            pc_offset: 0,
            op: Op::Push(reg),
        };
        let make_pop = |reg| NextOp {
            delay_cycles: 0,
            pc_offset: 0,
            op: Op::Pop(reg),
        };

        cpu.execute_op(&mut mem, &make_push(Reg16::AF));
        cpu.execute_op(&mut mem, &make_push(Reg16::BC));
        cpu.execute_op(&mut mem, &make_push(Reg16::DE));
        cpu.execute_op(&mut mem, &make_push(Reg16::HL));

        cpu.execute_op(&mut mem, &make_pop(Reg16::AF));
        cpu.execute_op(&mut mem, &make_pop(Reg16::BC));
        cpu.execute_op(&mut mem, &make_pop(Reg16::DE));
        cpu.execute_op(&mut mem, &make_pop(Reg16::HL));

        // Bottom 4 bits of F read-only, always zero.
        assert_eq!(cpu.regs.read16(Reg16::AF), 0x15F0);
        assert_eq!(cpu.regs.read16(Reg16::BC), 0x14FF);
        assert_eq!(cpu.regs.read16(Reg16::DE), 0x13FF);
        assert_eq!(cpu.regs.read16(Reg16::HL), 0x12F0);
    }

    #[test]
    fn swap() {
        let mut cpu = LR25902::new();
        let mut mem = Memory::new(vec![0; 0x100], vec![0; 0x1000]);

        cpu.regs.set8(Reg8::C, 0x12);

        let op = Alu8Op {
            op: Alu8::Swap,
            dest: Alu8Data::Reg(Reg8::C),
            y: Alu8Data::Ignore,
        };

        cpu.execute_alu8(&op, &mut mem);

        assert_eq!(cpu.regs.read8(Reg8::C), 0x21);
    }

    #[test]
    fn decimal_adjust_after_add() {
        let mut cpu = LR25902::new();
        let mut mem = Memory::new(vec![0; 0x100], vec![0; 0x1000]);

        let add = Alu8Op {
            op: Alu8::Add,
            dest: Alu8Data::Reg(Reg8::A),
            y: Alu8Data::Reg(Reg8::B),
        };

        let daa = Alu8Op {
            op: Alu8::DecimalAdjust,
            dest: Alu8Data::Reg(Reg8::A),
            y: Alu8Data::Ignore,
        };

        // Basic
        cpu.regs.set8(Reg8::A, 0xAA);
        cpu.execute_alu8(&daa, &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 0x10);
        assert_eq!(cpu.regs.read_flag(Flag::Carry), true);

        // Add two BCD numbers, without half-carry, or needing to adjust
        cpu.regs.set8(Reg8::A, 0x22);
        cpu.regs.set8(Reg8::B, 0x22);
        cpu.execute_alu8(&add, &mut mem);
        cpu.execute_alu8(&daa, &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 0x44);

        // Add two BCD numbers, need to adjust
        cpu.regs.set8(Reg8::A, 0x46);
        cpu.regs.set8(Reg8::B, 0x46);
        cpu.execute_alu8(&add, &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 0x8C);
        cpu.execute_alu8(&daa, &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 0x92);

        // Add two BCD numbers, have half-carry out of lower nibble
        cpu.regs.set8(Reg8::A, 0x18);
        cpu.regs.set8(Reg8::B, 0x18);
        cpu.execute_alu8(&add, &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 0x30);
        assert_eq!(cpu.regs.read_flag(Flag::HalfCarry), true);
        cpu.execute_alu8(&daa, &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 0x36);

        // Add two BCD numbers, have carry out of upper nibble
        cpu.regs.set8(Reg8::A, 0x70);
        cpu.regs.set8(Reg8::B, 0x70);
        cpu.execute_alu8(&add, &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 0xE0);
        assert_eq!(cpu.regs.read_flag(Flag::Carry), false);
        cpu.execute_alu8(&daa, &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 0x40);
        assert_eq!(cpu.regs.read_flag(Flag::Carry), true);
    }

    #[test]
    fn decimal_adjust_after_sub() {
        let mut cpu = LR25902::new();
        let mut mem = Memory::new(vec![0; 0x100], vec![0; 0x1000]);

        let sub = Alu8Op {
            op: Alu8::Sub,
            dest: Alu8Data::Reg(Reg8::A),
            y: Alu8Data::Reg(Reg8::B),
        };

        let daa = Alu8Op {
            op: Alu8::DecimalAdjust,
            dest: Alu8Data::Reg(Reg8::A),
            y: Alu8Data::Ignore,
        };

        // Sub two BCD numbers, without half-carry, or needing to adjust
        cpu.regs.set8(Reg8::A, 0x33);
        cpu.regs.set8(Reg8::B, 0x11);
        cpu.execute_alu8(&sub, &mut mem);
        cpu.execute_alu8(&daa, &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 0x22);

        // Sub two BCD numbers, need to adjust lower nibble
        cpu.regs.set8(Reg8::A, 0x20);
        cpu.regs.set8(Reg8::B, 0x04);
        cpu.execute_alu8(&sub, &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 0x1C);
        assert_eq!(cpu.regs.read_flag(Flag::Subtract), true);
        assert_eq!(cpu.regs.read_flag(Flag::HalfCarry), true);
        cpu.execute_alu8(&daa, &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 0x16);

        // Dec BCD numbers, need lots of carry
        cpu.regs.set8(Reg8::A, 0x00);
        cpu.regs.set8(Reg8::B, 0x01);
        cpu.execute_alu8(
            &Alu8Op {
                op: Alu8::Decrement,
                dest: Alu8Data::Reg(Reg8::A),
                y: Alu8Data::Ignore,
            },
            &mut mem,
        );
        assert_eq!(cpu.regs.read8(Reg8::A), 0xFF);
        assert_eq!(cpu.regs.read_flag(Flag::Subtract), true);
        assert_eq!(cpu.regs.read_flag(Flag::HalfCarry), true);
        cpu.execute_alu8(&daa, &mut mem);
        assert_eq!(cpu.regs.read8(Reg8::A), 0xF9);
    }
}
