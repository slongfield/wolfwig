use mem::model::Memory;
use std::fmt;

use cpu::registers::Flag::{self, Carry, NotCarry, NotZero, Zero};
use cpu::registers::Reg16::{self, AF, BC, DE, HL, SP};
use cpu::registers::Reg8::{self, A, B, C, D, E, H, L};
use util;

#[derive(Debug)]
pub enum Address {
    Register16(Reg16),
    Immediate16(u16),
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Address::Immediate16(val) => write!(f, "0x{:X}", val),
            Address::Register16(reg) => write!(f, "{}", reg),
        }
    }
}

///! Op
/// TODO(slongfield): Encode the microops that make up these instructions, and the flags that
/// they affect. Right now, mostly just doing this to display the instructions.
#[derive(Debug)]
pub enum Op {
    Alu8(Alu8Op),
    Alu16(Alu16Op),
    Call(u16),
    ConditionalCall(Flag, u16),
    ConditionalJump(Flag, u16),
    ConditionalJumpRelative(Flag, u16),
    ConditionalReturn(Flag),
    DisableInterrupts,
    EnableInterrupts,
    Halt,
    Jump(Address),
    JumpRelative(u16),
    Load(Reg8, Address),
    LoadAndDecrement(Reg8, Address),
    LoadAndIncrement(Reg8, Address),
    Move(Reg8, Reg8),
    Nop,
    Pop(Reg16),
    Push(Reg16),
    ReadIO(u8), // Reads to A from 0xFF00+u8
    ReadIOC,    // Reads A from 0xFF00+C
    Reset(u16),
    Return,
    ReturnAndEnableInterrupts,
    Set(Reg8, u8),
    SetIO(u8), // Sets from A from 0xFF00+u8
    SetIOC,    // Sets A from OxFF00+C
    SetWide(Reg16, u16),
    SetAddr(Reg16, u8),
    Stop,
    Store(Address, Reg8),
    StoreAndDecrement(Address, Reg8),
    StoreAndIncrement(Address, Reg8),
    Unknown(u8),
    WideStore(Address, Reg16),
}

// TODO(slongfield): Refactor this a bit.
// Make Jump operations their own kind of Op, with taken/not taken latencies.
impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Op::Alu8(op) => write!(f, "{}", op),
            Op::Alu16(op) => write!(f, "{}", op),
            Op::Call(address) => write!(f, "CALL 0x{:X}", address),
            Op::ConditionalCall(flag, address) => write!(f, "CALL {},0x{:X}", flag, address),
            Op::ConditionalJump(flag, address) => write!(f, "JP {},({})", flag, address),
            Op::ConditionalJumpRelative(flag, address) => write!(f, "JR {},0x{:X}", flag, address),
            Op::ConditionalReturn(flag) => write!(f, "RET {}", flag),
            Op::DisableInterrupts => write!(f, "DI"),
            Op::EnableInterrupts => write!(f, "EI"),
            Op::Halt => write!(f, "HALT"),
            Op::Jump(address) => write!(f, "JP ({})", address),
            Op::JumpRelative(address) => write!(f, "JR ({})", address),
            Op::Load(dest, addr) => write!(f, "LD {} ({})", dest, addr),
            Op::LoadAndDecrement(dest, addr) => write!(f, "LD {} ({}-)", dest, addr),
            Op::LoadAndIncrement(dest, addr) => write!(f, "LD {} ({}+)", dest, addr),
            Op::Move(src, dest) => write!(f, "LD {} {}", src, dest),
            Op::Nop => write!(f, "NOP"),
            Op::Pop(reg) => write!(f, "POP {}", reg),
            Op::Push(reg) => write!(f, "PUSH {}", reg),
            Op::ReadIO(offset) => write!(f, "LD 0xFF00+0x{:X}", offset),
            Op::ReadIOC => write!(f, "LD A,(FF00+C)"),
            Op::Reset(offset) => write!(f, "RST {:X}H", offset),
            Op::Return => write!(f, "RET"),
            Op::ReturnAndEnableInterrupts => write!(f, "RETI"),
            Op::Set(dest, val) => write!(f, "LD {} 0x{:X}", dest, val),
            Op::SetIO(offset) => write!(f, "LD 0xFF00+0x{:X},A", offset),
            Op::SetIOC => write!(f, "LD (FF00+C),A"),
            Op::SetWide(dest, val) => write!(f, "LD {} 0x{:X}", dest, val),
            Op::SetAddr(dest, val) => write!(f, "LD ({}) 0x{:X}", dest, val),
            Op::Stop => write!(f, "STOP"),
            Op::Store(addr, src) => write!(f, "LD ({}) {}", addr, src),
            Op::StoreAndDecrement(addr, src) => write!(f, "LD ({}-) {}", addr, src),
            Op::StoreAndIncrement(addr, src) => write!(f, "LD ({}+) {}", addr, src),
            Op::WideStore(addr, src) => write!(f, "LD ({}) {}", addr, src),
            Op::Unknown(code) => write!(f, "Don't know how to display: 0x{:X}", code),
        }
    }
}

#[derive(Debug)]
pub enum Alu8Data {
    Reg(Reg8),
    Imm(u8),
    Addr(Reg16),
    Ignore,
}

impl fmt::Display for Alu8Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Alu8Data::Reg(reg) => write!(f, "{}", reg),
            Alu8Data::Imm(data) => write!(f, "0x{:02X}", data),
            Alu8Data::Addr(reg) => write!(f, "({})", reg),
            Alu8Data::Ignore => write!(f, "???"),
        }
    }
}

#[derive(Debug)]
pub enum Alu8 {
    Add,
    AddWithCarry,
    And,
    ClearCarryFlag,
    Compare,
    Complement,
    DecimalAdjust,
    Decrement,
    Increment,
    Or,
    ResetBit,
    RotateLeft,
    RotateLeftCarry,
    RotateRight,
    RotateRightCarry,
    SetBit,
    SetCarryFlag,
    ShiftLeftArithmetic,
    ShiftRightArithmetic,
    ShiftRightLogical,
    Sub,
    SubWithCarry,
    Swap,
    TestBit,
    Unknown,
    Xor,
}

#[derive(Debug)]
pub struct Alu8Op {
    pub op: Alu8,
    pub dest: Alu8Data,
    pub y: Alu8Data,
}

impl Alu8Op {
    fn add(y: Alu8Data) -> Self {
        Self {
            op: Alu8::Add,
            dest: Alu8Data::Reg(A),
            y,
        }
    }

    fn add_with_carry(y: Alu8Data) -> Self {
        Self {
            op: Alu8::AddWithCarry,
            dest: Alu8Data::Reg(A),
            y,
        }
    }

    fn and(y: Alu8Data) -> Self {
        Self {
            op: Alu8::And,
            dest: Alu8Data::Reg(A),
            y,
        }
    }

    fn clear_carry_flag() -> Self {
        Self {
            op: Alu8::ClearCarryFlag,
            // No actual dest.
            dest: Alu8Data::Reg(A),
            y: Alu8Data::Ignore,
        }
    }

    fn compare(y: Alu8Data) -> Self {
        Self {
            op: Alu8::Compare,
            dest: Alu8Data::Reg(A),
            y,
        }
    }

    fn complement() -> Self {
        Self {
            op: Alu8::Complement,
            dest: Alu8Data::Reg(A),
            y: Alu8Data::Ignore,
        }
    }

    fn decimal_adjust() -> Self {
        Self {
            op: Alu8::DecimalAdjust,
            dest: Alu8Data::Reg(A),
            y: Alu8Data::Ignore,
        }
    }

    fn decrement(dest: Alu8Data) -> Self {
        Self {
            op: Alu8::Decrement,
            dest,
            y: Alu8Data::Ignore,
        }
    }

    fn increment(dest: Alu8Data) -> Self {
        Self {
            op: Alu8::Increment,
            dest,
            y: Alu8Data::Ignore,
        }
    }

    fn or(y: Alu8Data) -> Self {
        Self {
            op: Alu8::Or,
            dest: Alu8Data::Reg(A),
            y,
        }
    }

    fn reset_bit(dest: Alu8Data, bit: u8) -> Self {
        Self {
            op: Alu8::ResetBit,
            dest,
            y: Alu8Data::Imm(bit),
        }
    }

    fn rotate_left(dest: Alu8Data) -> Self {
        Self {
            op: Alu8::RotateLeft,
            dest,
            y: Alu8Data::Ignore,
        }
    }

    fn rotate_left_carry(dest: Alu8Data) -> Self {
        Self {
            op: Alu8::RotateLeftCarry,
            dest,
            y: Alu8Data::Ignore,
        }
    }

    fn rotate_right(dest: Alu8Data) -> Self {
        Self {
            op: Alu8::RotateRight,
            dest,
            y: Alu8Data::Ignore,
        }
    }

    fn rotate_right_carry(dest: Alu8Data) -> Self {
        Self {
            op: Alu8::RotateRightCarry,
            dest,
            y: Alu8Data::Ignore,
        }
    }

    fn set_bit(dest: Alu8Data, bit: u8) -> Self {
        Self {
            op: Alu8::SetBit,
            dest,
            y: Alu8Data::Imm(bit),
        }
    }

    fn set_carry_flag() -> Self {
        Self {
            op: Alu8::SetCarryFlag,
            // No actual dest.
            dest: Alu8Data::Reg(A),
            y: Alu8Data::Ignore,
        }
    }

    fn shift_left_arithmetic(dest: Alu8Data) -> Self {
        Self {
            op: Alu8::ShiftLeftArithmetic,
            dest,
            y: Alu8Data::Ignore,
        }
    }

    fn shift_right_arithmetic(dest: Alu8Data) -> Self {
        Self {
            op: Alu8::ShiftRightArithmetic,
            dest,
            y: Alu8Data::Ignore,
        }
    }

    fn shift_right_logical(dest: Alu8Data) -> Self {
        Self {
            op: Alu8::ShiftRightLogical,
            dest,
            y: Alu8Data::Ignore,
        }
    }

    fn sub(y: Alu8Data) -> Self {
        Self {
            op: Alu8::Sub,
            dest: Alu8Data::Reg(A),
            y,
        }
    }

    fn sub_with_carry(y: Alu8Data) -> Self {
        Self {
            op: Alu8::SubWithCarry,
            dest: Alu8Data::Reg(A),
            y,
        }
    }

    fn xor(y: Alu8Data) -> Self {
        Self {
            op: Alu8::Xor,
            dest: Alu8Data::Reg(A),
            y,
        }
    }

    fn swap(dest: Alu8Data) -> Self {
        Self {
            op: Alu8::Swap,
            dest,
            y: Alu8Data::Ignore,
        }
    }

    fn test_bit(dest: Alu8Data, bit: u8) -> Self {
        Self {
            op: Alu8::TestBit,
            dest,
            y: Alu8Data::Imm(bit),
        }
    }

    fn unknown() -> Self {
        Self {
            op: Alu8::Unknown,
            dest: Alu8Data::Reg(A),
            y: Alu8Data::Ignore,
        }
    }
}

impl fmt::Display for Alu8Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.op {
            Alu8::Add => write!(f, "ADD {},{}", self.dest, self.y),
            Alu8::AddWithCarry => write!(f, "ADC {},{}", self.dest, self.y),
            Alu8::And => write!(f, "AND {},{}", self.dest, self.y),
            Alu8::ClearCarryFlag => write!(f, "CCF"),
            Alu8::Compare => write!(f, "CP {},{}", self.dest, self.y),
            Alu8::Complement => write!(f, "CPL"),
            Alu8::DecimalAdjust => write!(f, "DAA"),
            Alu8::Decrement => write!(f, "DEC {}", self.dest),
            Alu8::Increment => write!(f, "INC {}", self.dest),
            Alu8::Or => write!(f, "OR {},{}", self.dest, self.y),
            Alu8::ResetBit => write!(f, "RES {},{}", self.y, self.dest),
            Alu8::RotateLeft => write!(f, "RL {}", self.dest),
            Alu8::RotateLeftCarry => write!(f, "RLC {}", self.dest),
            Alu8::RotateRight => write!(f, "RR {}", self.dest),
            Alu8::RotateRightCarry => write!(f, "RRC {}", self.dest),
            Alu8::SetBit => write!(f, "SET {},{}", self.y, self.dest),
            Alu8::SetCarryFlag => write!(f, "SCF"),
            Alu8::ShiftLeftArithmetic => write!(f, "SLA {}", self.dest),
            Alu8::ShiftRightArithmetic => write!(f, "SRA {}", self.dest),
            Alu8::ShiftRightLogical => write!(f, "SRL {}", self.dest),
            Alu8::Sub => write!(f, "SUB {},{}", self.dest, self.y),
            Alu8::SubWithCarry => write!(f, "SBC {},{}", self.dest, self.y),
            Alu8::Swap => write!(f, "SWAP"),
            Alu8::TestBit => write!(f, "TEST {},{}", self.y, self.dest),
            Alu8::Xor => write!(f, "XOR {},{}", self.dest, self.y),
            Alu8::Unknown => write!(f, "UNKNOWN ALU8!"),
        }
    }
}

#[derive(Debug)]
pub enum Alu16Data {
    Reg(Reg16),
    Imm(i8),
    Ignore,
}

#[derive(Debug)]
pub enum Alu16 {
    Add,
    Decrement,
    Increment,
    Move,
    MoveAndAdd,
    Unknown,
}

#[derive(Debug)]
pub struct Alu16Op {
    pub op: Alu16,
    pub dest: Reg16,
    pub y: Alu16Data,
    pub imm: i8,
}

impl Alu16Op {
    fn add(dest: Reg16, y: Reg16) -> Self {
        Self {
            op: Alu16::Add,
            dest,
            y: Alu16Data::Reg(y),
            imm: 0,
        }
    }

    fn add_imm(dest: Reg16, y: i8) -> Self {
        Self {
            op: Alu16::Add,
            dest,
            y: Alu16Data::Imm(y),
            imm: 0,
        }
    }

    fn dec(dest: Reg16) -> Self {
        Self {
            op: Alu16::Decrement,
            dest,
            y: Alu16Data::Ignore,
            imm: 0,
        }
    }

    fn inc(dest: Reg16) -> Self {
        Self {
            op: Alu16::Increment,
            dest,
            y: Alu16Data::Ignore,
            imm: 0,
        }
    }

    fn move_reg(dest: Reg16, src: Reg16) -> Self {
        Self {
            op: Alu16::Move,
            dest,
            y: Alu16Data::Reg(src),
            imm: 0,
        }
    }

    fn move_and_add(dest: Reg16, src: Reg16, imm: i8) -> Self {
        Self {
            op: Alu16::MoveAndAdd,
            dest,
            y: Alu16Data::Reg(src),
            imm,
        }
    }

    fn unknown() -> Self {
        Self {
            op: Alu16::Unknown,
            dest: HL,
            y: Alu16Data::Ignore,
            imm: 0,
        }
    }
}

impl fmt::Display for Alu16Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.op {
            Alu16::Add => match self.y {
                Alu16Data::Reg(reg) => write!(f, "ADD {},{}", self.dest, reg),
                Alu16Data::Imm(data) => write!(f, "ADD {},{}", self.dest, data),
                _ => write!(f, "ADD {},???", self.dest),
            },
            Alu16::Increment => write!(f, "INC {}", self.dest),
            Alu16::Decrement => write!(f, "DEC {}", self.dest),
            Alu16::Move => write!(f, "LD {},{:?}", self.dest, self.y),
            Alu16::MoveAndAdd => write!(f, "LD {},{:?}+0x{:02X}", self.dest, self.y, self.imm),
            Alu16::Unknown => write!(f, "ALU16 ???"),
        }
    }
}

///! Decode takes the ROM and current PC, and returns the Op a that PC, as well as the number of
///! bytes in that op, and the number of cycles it runs for.
pub fn decode(rom: &Memory, pc: usize) -> (Op, usize, usize) {
    if let Some((op, size, time)) = decode_alu8(&rom, pc) {
        return (op, size, time);
    }
    if let Some((op, size, time)) = decode_alu16(&rom, pc) {
        return (op, size, time);
    }
    if let Some((op, size, time)) = decode_load(&rom, pc) {
        return (op, size, time);
    }
    if let Some((op, size, time)) = decode_jump(&rom, pc) {
        return (op, size, time);
    }
    match rom.read(pc) {
        0x00 => (Op::Nop, 1, 1),
        0x10 => (Op::Stop, 2, 1),
        0x76 => (Op::Halt, 1, 1),
        0xF3 => (Op::DisableInterrupts, 1, 1),
        0xFB => (Op::EnableInterrupts, 1, 1),
        0xCB => decode_extended(rom.read(pc + 1)),
        code => (Op::Unknown(code), 1, 0),
    }
}

///! Decode ALU operations.
fn decode_alu8(rom: &Memory, pc: usize) -> Option<(Op, usize, usize)> {
    let imm8 = rom.read(pc + 1);
    let inst = match rom.read(pc) {
        0x04 => (Alu8Op::increment(Alu8Data::Reg(B)), 1, 1),
        0x14 => (Alu8Op::increment(Alu8Data::Reg(D)), 1, 1),
        0x24 => (Alu8Op::increment(Alu8Data::Reg(H)), 1, 1),
        0x34 => (Alu8Op::increment(Alu8Data::Addr(HL)), 1, 3),
        0x0C => (Alu8Op::increment(Alu8Data::Reg(C)), 1, 1),
        0x1C => (Alu8Op::increment(Alu8Data::Reg(E)), 1, 1),
        0x2C => (Alu8Op::increment(Alu8Data::Reg(L)), 1, 1),
        0x3C => (Alu8Op::increment(Alu8Data::Reg(A)), 1, 1),

        0x05 => (Alu8Op::decrement(Alu8Data::Reg(B)), 1, 1),
        0x15 => (Alu8Op::decrement(Alu8Data::Reg(D)), 1, 1),
        0x25 => (Alu8Op::decrement(Alu8Data::Reg(H)), 1, 1),
        0x35 => (Alu8Op::decrement(Alu8Data::Addr(HL)), 1, 3),
        0x0D => (Alu8Op::decrement(Alu8Data::Reg(C)), 1, 1),
        0x1D => (Alu8Op::decrement(Alu8Data::Reg(E)), 1, 1),
        0x2D => (Alu8Op::decrement(Alu8Data::Reg(L)), 1, 1),
        0x3D => (Alu8Op::decrement(Alu8Data::Reg(A)), 1, 1),

        0x07 => (Alu8Op::rotate_left_carry(Alu8Data::Reg(A)), 1, 1),
        0x0F => (Alu8Op::rotate_right_carry(Alu8Data::Reg(A)), 1, 1),
        0x17 => (Alu8Op::rotate_left(Alu8Data::Reg(A)), 1, 1),
        0x1F => (Alu8Op::rotate_right(Alu8Data::Reg(A)), 1, 1),

        0x27 => (Alu8Op::decimal_adjust(), 1, 1),
        0x2F => (Alu8Op::complement(), 1, 1),

        0x37 => (Alu8Op::set_carry_flag(), 1, 1),
        0x3F => (Alu8Op::clear_carry_flag(), 1, 1),

        0x80 => (Alu8Op::add(Alu8Data::Reg(B)), 1, 1),
        0x81 => (Alu8Op::add(Alu8Data::Reg(C)), 1, 1),
        0x82 => (Alu8Op::add(Alu8Data::Reg(D)), 1, 1),
        0x83 => (Alu8Op::add(Alu8Data::Reg(E)), 1, 1),
        0x84 => (Alu8Op::add(Alu8Data::Reg(H)), 1, 1),
        0x85 => (Alu8Op::add(Alu8Data::Reg(L)), 1, 1),
        0x86 => (Alu8Op::add(Alu8Data::Addr(HL)), 1, 1),
        0x87 => (Alu8Op::add(Alu8Data::Reg(A)), 1, 2),
        0x88 => (Alu8Op::add_with_carry(Alu8Data::Reg(B)), 1, 1),
        0x89 => (Alu8Op::add_with_carry(Alu8Data::Reg(C)), 1, 1),
        0x8A => (Alu8Op::add_with_carry(Alu8Data::Reg(D)), 1, 1),
        0x8B => (Alu8Op::add_with_carry(Alu8Data::Reg(E)), 1, 1),
        0x8C => (Alu8Op::add_with_carry(Alu8Data::Reg(H)), 1, 1),
        0x8D => (Alu8Op::add_with_carry(Alu8Data::Reg(L)), 1, 1),
        0x8E => (Alu8Op::add_with_carry(Alu8Data::Addr(HL)), 1, 2),
        0x8F => (Alu8Op::add_with_carry(Alu8Data::Reg(A)), 1, 1),

        0x90 => (Alu8Op::sub(Alu8Data::Reg(B)), 1, 1),
        0x91 => (Alu8Op::sub(Alu8Data::Reg(C)), 1, 1),
        0x92 => (Alu8Op::sub(Alu8Data::Reg(D)), 1, 1),
        0x93 => (Alu8Op::sub(Alu8Data::Reg(E)), 1, 1),
        0x94 => (Alu8Op::sub(Alu8Data::Reg(H)), 1, 1),
        0x95 => (Alu8Op::sub(Alu8Data::Reg(L)), 1, 1),
        0x96 => (Alu8Op::sub(Alu8Data::Addr(HL)), 1, 2),
        0x97 => (Alu8Op::sub(Alu8Data::Reg(A)), 1, 1),
        0x98 => (Alu8Op::sub_with_carry(Alu8Data::Reg(B)), 1, 1),
        0x99 => (Alu8Op::sub_with_carry(Alu8Data::Reg(C)), 1, 1),
        0x9A => (Alu8Op::sub_with_carry(Alu8Data::Reg(D)), 1, 1),
        0x9B => (Alu8Op::sub_with_carry(Alu8Data::Reg(E)), 1, 1),
        0x9C => (Alu8Op::sub_with_carry(Alu8Data::Reg(H)), 1, 1),
        0x9D => (Alu8Op::sub_with_carry(Alu8Data::Reg(L)), 1, 1),
        0x9E => (Alu8Op::sub_with_carry(Alu8Data::Addr(HL)), 1, 2),
        0x9F => (Alu8Op::sub_with_carry(Alu8Data::Reg(A)), 1, 1),

        0xA0 => (Alu8Op::and(Alu8Data::Reg(B)), 1, 1),
        0xA1 => (Alu8Op::and(Alu8Data::Reg(C)), 1, 1),
        0xA2 => (Alu8Op::and(Alu8Data::Reg(D)), 1, 1),
        0xA3 => (Alu8Op::and(Alu8Data::Reg(E)), 1, 1),
        0xA4 => (Alu8Op::and(Alu8Data::Reg(H)), 1, 1),
        0xA5 => (Alu8Op::and(Alu8Data::Reg(L)), 1, 1),
        0xA6 => (Alu8Op::and(Alu8Data::Addr(HL)), 1, 2),
        0xA7 => (Alu8Op::and(Alu8Data::Reg(A)), 1, 1),

        0xA8 => (Alu8Op::xor(Alu8Data::Reg(B)), 1, 1),
        0xA9 => (Alu8Op::xor(Alu8Data::Reg(C)), 1, 1),
        0xAA => (Alu8Op::xor(Alu8Data::Reg(D)), 1, 1),
        0xAB => (Alu8Op::xor(Alu8Data::Reg(E)), 1, 1),
        0xAC => (Alu8Op::xor(Alu8Data::Reg(H)), 1, 1),
        0xAD => (Alu8Op::xor(Alu8Data::Reg(L)), 1, 1),
        0xAE => (Alu8Op::xor(Alu8Data::Addr(HL)), 1, 2),
        0xAF => (Alu8Op::xor(Alu8Data::Reg(A)), 1, 1),

        0xB0 => (Alu8Op::or(Alu8Data::Reg(B)), 1, 1),
        0xB1 => (Alu8Op::or(Alu8Data::Reg(C)), 1, 1),
        0xB2 => (Alu8Op::or(Alu8Data::Reg(D)), 1, 1),
        0xB3 => (Alu8Op::or(Alu8Data::Reg(E)), 1, 1),
        0xB4 => (Alu8Op::or(Alu8Data::Reg(H)), 1, 1),
        0xB5 => (Alu8Op::or(Alu8Data::Reg(L)), 1, 1),
        0xB6 => (Alu8Op::or(Alu8Data::Addr(HL)), 1, 2),
        0xB7 => (Alu8Op::or(Alu8Data::Reg(A)), 1, 1),

        0xB8 => (Alu8Op::compare(Alu8Data::Reg(B)), 1, 1),
        0xB9 => (Alu8Op::compare(Alu8Data::Reg(C)), 1, 1),
        0xBA => (Alu8Op::compare(Alu8Data::Reg(D)), 1, 1),
        0xBB => (Alu8Op::compare(Alu8Data::Reg(E)), 1, 1),
        0xBC => (Alu8Op::compare(Alu8Data::Reg(H)), 1, 1),
        0xBD => (Alu8Op::compare(Alu8Data::Reg(L)), 1, 1),
        0xBE => (Alu8Op::compare(Alu8Data::Addr(HL)), 1, 2),
        0xBF => (Alu8Op::compare(Alu8Data::Reg(A)), 1, 1),

        0xC6 => (Alu8Op::add(Alu8Data::Imm(imm8)), 2, 1),
        0xD6 => (Alu8Op::sub(Alu8Data::Imm(imm8)), 2, 1),
        0xE6 => (Alu8Op::and(Alu8Data::Imm(imm8)), 2, 1),
        0xF6 => (Alu8Op::or(Alu8Data::Imm(imm8)), 2, 1),
        0xCE => (Alu8Op::add_with_carry(Alu8Data::Imm(imm8)), 2, 1),
        0xDE => (Alu8Op::sub_with_carry(Alu8Data::Imm(imm8)), 2, 1),
        0xEE => (Alu8Op::xor(Alu8Data::Imm(imm8)), 2, 1),
        0xFE => (Alu8Op::compare(Alu8Data::Imm(imm8)), 2, 1),

        _ => (Alu8Op::unknown(), 0, 0),
    };
    match inst {
        (
            Alu8Op {
                op: Alu8::Unknown, ..
            },
            _,
            _,
        ) => None,
        (op, size, time) => Some((Op::Alu8(op), size, time)),
    }
}

///! Decode ALU operations.
fn decode_alu16(rom: &Memory, pc: usize) -> Option<(Op, usize, usize)> {
    let inst = match rom.read(pc) {
        0x03 => (Alu16Op::inc(BC), 1, 1),
        0x13 => (Alu16Op::inc(DE), 1, 1),
        0x23 => (Alu16Op::inc(HL), 1, 1),
        0x33 => (Alu16Op::inc(SP), 1, 1),

        0x09 => (Alu16Op::add(HL, BC), 1, 2),
        0x19 => (Alu16Op::add(HL, DE), 1, 2),
        0x29 => (Alu16Op::add(HL, HL), 1, 2),
        0x39 => (Alu16Op::add(HL, SP), 1, 2),

        0x0B => (Alu16Op::dec(BC), 1, 2),
        0x1B => (Alu16Op::dec(DE), 1, 2),
        0x2B => (Alu16Op::dec(HL), 1, 2),
        0x3B => (Alu16Op::dec(SP), 1, 2),

        0xE8 => (Alu16Op::add_imm(SP, rom.read(pc + 1) as i8), 2, 4),

        0xF8 => (Alu16Op::move_and_add(HL, SP, rom.read(pc + 1) as i8), 2, 3),

        0xF9 => (Alu16Op::move_reg(SP, HL), 1, 2),

        _ => (Alu16Op::unknown(), 0, 0),
    };
    match inst {
        (
            Alu16Op {
                op: Alu16::Unknown, ..
            },
            _,
            _,
        ) => None,
        (op, size, time) => Some((Op::Alu16(op), size, time)),
    }
}

///! Decode move, load, and store operations.
fn decode_load(rom: &Memory, pc: usize) -> Option<(Op, usize, usize)> {
    let imm16 = util::bytes_to_u16(&[rom.read(pc + 2), rom.read(pc + 1)]);
    let imm8 = rom.read(pc + 1);
    let inst = match rom.read(pc) {
        0x01 => (Op::SetWide(BC, imm16), 3, 3),
        0x11 => (Op::SetWide(DE, imm16), 3, 3),
        0x21 => (Op::SetWide(HL, imm16), 3, 3),
        0x31 => (Op::SetWide(SP, imm16), 3, 3),

        0x02 => (Op::Store(Address::Register16(BC), A), 1, 2),
        0x12 => (Op::Store(Address::Register16(DE), A), 1, 2),
        0x22 => (Op::StoreAndIncrement(Address::Register16(HL), A), 1, 2),
        0x32 => (Op::StoreAndDecrement(Address::Register16(HL), A), 1, 2),

        0x06 => (Op::Set(B, imm8), 2, 2),
        0x16 => (Op::Set(D, imm8), 2, 2),
        0x26 => (Op::Set(H, imm8), 2, 2),
        0x36 => (Op::SetAddr(HL, imm8), 2, 2),

        0x08 => (Op::WideStore(Address::Immediate16(imm16), SP), 3, 5),

        0x0E => (Op::Set(C, imm8), 2, 2),
        0x1E => (Op::Set(E, imm8), 2, 2),
        0x2E => (Op::Set(L, imm8), 2, 2),
        0x3E => (Op::Set(A, imm8), 2, 2),

        0x0A => (Op::Load(A, Address::Register16(BC)), 1, 2),
        0x1A => (Op::Load(A, Address::Register16(DE)), 1, 2),
        0xFA => (Op::Load(A, Address::Immediate16(imm16)), 3, 2),
        0x2A => (Op::LoadAndIncrement(A, Address::Register16(HL)), 1, 2),
        0x3A => (Op::LoadAndDecrement(A, Address::Register16(HL)), 1, 2),

        0x40 => (Op::Move(B, B), 1, 1),
        0x41 => (Op::Move(B, C), 1, 1),
        0x42 => (Op::Move(B, D), 1, 1),
        0x43 => (Op::Move(B, E), 1, 1),
        0x44 => (Op::Move(B, H), 1, 1),
        0x45 => (Op::Move(B, L), 1, 1),
        0x47 => (Op::Move(B, A), 1, 1),

        0x48 => (Op::Move(C, B), 1, 1),
        0x49 => (Op::Move(C, C), 1, 1),
        0x4A => (Op::Move(C, D), 1, 1),
        0x4B => (Op::Move(C, E), 1, 1),
        0x4C => (Op::Move(C, H), 1, 1),
        0x4D => (Op::Move(C, L), 1, 1),
        0x4F => (Op::Move(C, A), 1, 1),

        0x50 => (Op::Move(D, B), 1, 1),
        0x51 => (Op::Move(D, C), 1, 1),
        0x52 => (Op::Move(D, D), 1, 1),
        0x53 => (Op::Move(D, E), 1, 1),
        0x54 => (Op::Move(D, H), 1, 1),
        0x55 => (Op::Move(D, L), 1, 1),
        0x57 => (Op::Move(D, A), 1, 1),

        0x58 => (Op::Move(E, B), 1, 1),
        0x59 => (Op::Move(E, C), 1, 1),
        0x5A => (Op::Move(E, D), 1, 1),
        0x5B => (Op::Move(E, E), 1, 1),
        0x5C => (Op::Move(E, H), 1, 1),
        0x5D => (Op::Move(E, L), 1, 1),
        0x5F => (Op::Move(E, A), 1, 1),

        0x60 => (Op::Move(H, B), 1, 1),
        0x61 => (Op::Move(H, C), 1, 1),
        0x62 => (Op::Move(H, D), 1, 1),
        0x63 => (Op::Move(H, E), 1, 1),
        0x64 => (Op::Move(H, H), 1, 1),
        0x65 => (Op::Move(H, L), 1, 1),
        0x67 => (Op::Move(H, A), 1, 1),

        0x68 => (Op::Move(L, B), 1, 1),
        0x69 => (Op::Move(L, C), 1, 1),
        0x6A => (Op::Move(L, D), 1, 1),
        0x6B => (Op::Move(L, E), 1, 1),
        0x6C => (Op::Move(L, H), 1, 1),
        0x6D => (Op::Move(L, L), 1, 1),
        0x6F => (Op::Move(L, A), 1, 1),

        0x78 => (Op::Move(A, B), 1, 1),
        0x79 => (Op::Move(A, C), 1, 1),
        0x7A => (Op::Move(A, D), 1, 1),
        0x7B => (Op::Move(A, E), 1, 1),
        0x7C => (Op::Move(A, H), 1, 1),
        0x7D => (Op::Move(A, L), 1, 1),
        0x7F => (Op::Move(A, A), 1, 1),

        0x46 => (Op::Load(B, Address::Register16(HL)), 1, 2),
        0x4E => (Op::Load(C, Address::Register16(HL)), 1, 2),
        0x56 => (Op::Load(D, Address::Register16(HL)), 1, 2),
        0x5E => (Op::Load(E, Address::Register16(HL)), 1, 2),
        0x66 => (Op::Load(H, Address::Register16(HL)), 1, 2),
        0x6E => (Op::Load(L, Address::Register16(HL)), 1, 2),
        0x7E => (Op::Load(A, Address::Register16(HL)), 1, 2),

        0x70 => (Op::Store(Address::Register16(HL), B), 1, 2),
        0x71 => (Op::Store(Address::Register16(HL), C), 1, 2),
        0x72 => (Op::Store(Address::Register16(HL), D), 1, 2),
        0x73 => (Op::Store(Address::Register16(HL), E), 1, 2),
        0x74 => (Op::Store(Address::Register16(HL), H), 1, 2),
        0x75 => (Op::Store(Address::Register16(HL), L), 1, 2),
        0x77 => (Op::Store(Address::Register16(HL), A), 1, 2),
        0xEA => (Op::Store(Address::Immediate16(imm16), A), 3, 2),

        0xE0 => (Op::SetIO(imm8), 2, 3),
        0xE2 => (Op::SetIOC, 1, 3),
        0xF0 => (Op::ReadIO(imm8), 2, 3),
        0xF2 => (Op::ReadIOC, 1, 3),

        0xC1 => (Op::Pop(BC), 1, 3),
        0xD1 => (Op::Pop(DE), 1, 3),
        0xE1 => (Op::Pop(HL), 1, 3),
        0xF1 => (Op::Pop(AF), 1, 3),
        0xC5 => (Op::Push(BC), 1, 4),
        0xD5 => (Op::Push(DE), 1, 4),
        0xE5 => (Op::Push(HL), 1, 4),
        0xF5 => (Op::Push(AF), 1, 4),

        code => (Op::Unknown(code), 0, 0),
    };
    match inst {
        (Op::Unknown(_), _, _) => None,
        (op, size, time) => Some((op, size, time)),
    }
}

///! Decode ALU operations.
fn decode_jump(rom: &Memory, pc: usize) -> Option<(Op, usize, usize)> {
    let dest16 = util::bytes_to_u16(&[rom.read(pc + 2), rom.read(pc + 1)]);
    let relative_dest = (((pc + 2) as isize) + ((rom.read(pc + 1) as i8) as isize)) as u16;
    let inst = match rom.read(pc) {
        // Conditional jumps take an extra cycle if they're taken.
        // TODO(slongfield) Annotate this.
        0x20 => (Op::ConditionalJumpRelative(NotZero, relative_dest), 2, 2),
        0x30 => (Op::ConditionalJumpRelative(NotCarry, relative_dest), 2, 2),
        0x28 => (Op::ConditionalJumpRelative(Zero, relative_dest), 2, 2),
        0x38 => (Op::ConditionalJumpRelative(Carry, relative_dest), 2, 2),
        0x18 => (Op::JumpRelative(relative_dest), 2, 3),
        0xC2 => (Op::ConditionalJump(NotZero, dest16), 3, 3),
        0xD2 => (Op::ConditionalJump(NotCarry, dest16), 3, 3),
        0xCA => (Op::ConditionalJump(Zero, dest16), 3, 3),
        0xDA => (Op::ConditionalJump(Carry, dest16), 3, 3),
        0xC3 => (Op::Jump(Address::Immediate16(dest16)), 3, 4),
        0xE9 => (Op::Jump(Address::Register16(HL)), 1, 4),
        0xC7 => (Op::Reset(0x0), 1, 4),
        0xD7 => (Op::Reset(0x10), 1, 4),
        0xE7 => (Op::Reset(0x20), 1, 4),
        0xF7 => (Op::Reset(0x30), 1, 4),
        0xCF => (Op::Reset(0x8), 1, 4),
        0xDF => (Op::Reset(0x18), 1, 4),
        0xEF => (Op::Reset(0x28), 1, 4),
        0xFF => (Op::Reset(0x38), 1, 4),
        // Conditional returns take 3 extra cycles if they're taken.
        0xC0 => (Op::ConditionalReturn(NotZero), 1, 2),
        0xD0 => (Op::ConditionalReturn(NotCarry), 1, 2),
        0xC8 => (Op::ConditionalReturn(Zero), 1, 2),
        0xD8 => (Op::ConditionalReturn(Carry), 1, 2),
        0xC9 => (Op::Return, 1, 4),
        0xD9 => (Op::ReturnAndEnableInterrupts, 1, 4),
        // Conditional calls take an extra 3 cycles.
        0xC4 => (Op::ConditionalCall(NotZero, dest16), 3, 3),
        0xD4 => (Op::ConditionalCall(NotCarry, dest16), 3, 3),
        0xCC => (Op::ConditionalCall(Zero, dest16), 3, 3),
        0xDC => (Op::ConditionalCall(Carry, dest16), 3, 3),
        0xCD => (Op::Call(dest16), 3, 3),

        code => (Op::Unknown(code), 0, 0),
    };
    match inst {
        (Op::Unknown(_), _, _) => None,
        (op, size, time) => Some((op, size, time)),
    }
}

///! Decode prefix 0xCB extended ops
fn decode_extended(opcode: u8) -> (Op, usize, usize) {
    let (alu_op, time) = match opcode {
        0x00 => (Alu8Op::rotate_left_carry(Alu8Data::Reg(B)), 2),
        0x01 => (Alu8Op::rotate_left_carry(Alu8Data::Reg(C)), 2),
        0x02 => (Alu8Op::rotate_left_carry(Alu8Data::Reg(D)), 2),
        0x03 => (Alu8Op::rotate_left_carry(Alu8Data::Reg(E)), 2),
        0x04 => (Alu8Op::rotate_left_carry(Alu8Data::Reg(H)), 2),
        0x05 => (Alu8Op::rotate_left_carry(Alu8Data::Reg(L)), 2),
        0x06 => (Alu8Op::rotate_left_carry(Alu8Data::Addr(HL)), 4),
        0x07 => (Alu8Op::rotate_left_carry(Alu8Data::Reg(A)), 2),

        0x08 => (Alu8Op::rotate_right_carry(Alu8Data::Reg(B)), 2),
        0x09 => (Alu8Op::rotate_right_carry(Alu8Data::Reg(C)), 2),
        0x0A => (Alu8Op::rotate_right_carry(Alu8Data::Reg(D)), 2),
        0x0B => (Alu8Op::rotate_right_carry(Alu8Data::Reg(E)), 2),
        0x0C => (Alu8Op::rotate_right_carry(Alu8Data::Reg(H)), 2),
        0x0D => (Alu8Op::rotate_right_carry(Alu8Data::Reg(L)), 2),
        0x0E => (Alu8Op::rotate_right_carry(Alu8Data::Addr(HL)), 4),
        0x0F => (Alu8Op::rotate_right_carry(Alu8Data::Reg(A)), 2),

        0x10 => (Alu8Op::rotate_left(Alu8Data::Reg(B)), 2),
        0x11 => (Alu8Op::rotate_left(Alu8Data::Reg(C)), 2),
        0x12 => (Alu8Op::rotate_left(Alu8Data::Reg(D)), 2),
        0x13 => (Alu8Op::rotate_left(Alu8Data::Reg(E)), 2),
        0x14 => (Alu8Op::rotate_left(Alu8Data::Reg(H)), 2),
        0x15 => (Alu8Op::rotate_left(Alu8Data::Reg(L)), 2),
        0x16 => (Alu8Op::rotate_left(Alu8Data::Addr(HL)), 4),
        0x17 => (Alu8Op::rotate_left(Alu8Data::Reg(A)), 2),

        0x18 => (Alu8Op::rotate_right(Alu8Data::Reg(B)), 2),
        0x19 => (Alu8Op::rotate_right(Alu8Data::Reg(C)), 2),
        0x1A => (Alu8Op::rotate_right(Alu8Data::Reg(D)), 2),
        0x1B => (Alu8Op::rotate_right(Alu8Data::Reg(E)), 2),
        0x1C => (Alu8Op::rotate_right(Alu8Data::Reg(H)), 2),
        0x1D => (Alu8Op::rotate_right(Alu8Data::Reg(L)), 2),
        0x1E => (Alu8Op::rotate_right(Alu8Data::Addr(HL)), 4),
        0x1F => (Alu8Op::rotate_right(Alu8Data::Reg(A)), 2),

        0x20 => (Alu8Op::shift_left_arithmetic(Alu8Data::Reg(B)), 2),
        0x21 => (Alu8Op::shift_left_arithmetic(Alu8Data::Reg(C)), 2),
        0x22 => (Alu8Op::shift_left_arithmetic(Alu8Data::Reg(D)), 2),
        0x23 => (Alu8Op::shift_left_arithmetic(Alu8Data::Reg(E)), 2),
        0x24 => (Alu8Op::shift_left_arithmetic(Alu8Data::Reg(H)), 2),
        0x25 => (Alu8Op::shift_left_arithmetic(Alu8Data::Reg(L)), 2),
        0x26 => (Alu8Op::shift_left_arithmetic(Alu8Data::Addr(HL)), 4),
        0x27 => (Alu8Op::shift_left_arithmetic(Alu8Data::Reg(A)), 2),

        0x28 => (Alu8Op::shift_right_arithmetic(Alu8Data::Reg(B)), 2),
        0x29 => (Alu8Op::shift_right_arithmetic(Alu8Data::Reg(C)), 2),
        0x2A => (Alu8Op::shift_right_arithmetic(Alu8Data::Reg(D)), 2),
        0x2B => (Alu8Op::shift_right_arithmetic(Alu8Data::Reg(E)), 2),
        0x2C => (Alu8Op::shift_right_arithmetic(Alu8Data::Reg(H)), 2),
        0x2D => (Alu8Op::shift_right_arithmetic(Alu8Data::Reg(L)), 2),
        0x2E => (Alu8Op::shift_right_arithmetic(Alu8Data::Addr(HL)), 4),
        0x2F => (Alu8Op::shift_right_arithmetic(Alu8Data::Reg(A)), 2),

        0x30 => (Alu8Op::swap(Alu8Data::Reg(B)), 2),
        0x31 => (Alu8Op::swap(Alu8Data::Reg(C)), 2),
        0x32 => (Alu8Op::swap(Alu8Data::Reg(D)), 2),
        0x33 => (Alu8Op::swap(Alu8Data::Reg(E)), 2),
        0x34 => (Alu8Op::swap(Alu8Data::Reg(H)), 2),
        0x35 => (Alu8Op::swap(Alu8Data::Reg(L)), 2),
        0x36 => (Alu8Op::swap(Alu8Data::Addr(HL)), 4),
        0x37 => (Alu8Op::swap(Alu8Data::Reg(A)), 2),

        0x38 => (Alu8Op::shift_right_logical(Alu8Data::Reg(B)), 2),
        0x39 => (Alu8Op::shift_right_logical(Alu8Data::Reg(C)), 2),
        0x3A => (Alu8Op::shift_right_logical(Alu8Data::Reg(D)), 2),
        0x3B => (Alu8Op::shift_right_logical(Alu8Data::Reg(E)), 2),
        0x3C => (Alu8Op::shift_right_logical(Alu8Data::Reg(H)), 2),
        0x3D => (Alu8Op::shift_right_logical(Alu8Data::Reg(L)), 2),
        0x3E => (Alu8Op::shift_right_logical(Alu8Data::Addr(HL)), 4),
        0x3F => (Alu8Op::shift_right_logical(Alu8Data::Reg(A)), 2),

        0x40 => (Alu8Op::test_bit(Alu8Data::Reg(B), 0), 2),
        0x41 => (Alu8Op::test_bit(Alu8Data::Reg(C), 0), 2),
        0x42 => (Alu8Op::test_bit(Alu8Data::Reg(D), 0), 2),
        0x43 => (Alu8Op::test_bit(Alu8Data::Reg(E), 0), 2),
        0x44 => (Alu8Op::test_bit(Alu8Data::Reg(H), 0), 2),
        0x45 => (Alu8Op::test_bit(Alu8Data::Reg(L), 0), 2),
        0x46 => (Alu8Op::test_bit(Alu8Data::Addr(HL), 0), 4),
        0x47 => (Alu8Op::test_bit(Alu8Data::Reg(A), 0), 2),

        0x48 => (Alu8Op::test_bit(Alu8Data::Reg(B), 1), 2),
        0x49 => (Alu8Op::test_bit(Alu8Data::Reg(C), 1), 2),
        0x4A => (Alu8Op::test_bit(Alu8Data::Reg(D), 1), 2),
        0x4B => (Alu8Op::test_bit(Alu8Data::Reg(E), 1), 2),
        0x4C => (Alu8Op::test_bit(Alu8Data::Reg(H), 1), 2),
        0x4D => (Alu8Op::test_bit(Alu8Data::Reg(L), 1), 2),
        0x4E => (Alu8Op::test_bit(Alu8Data::Addr(HL), 1), 4),
        0x4F => (Alu8Op::test_bit(Alu8Data::Reg(A), 1), 2),

        0x50 => (Alu8Op::test_bit(Alu8Data::Reg(B), 2), 2),
        0x51 => (Alu8Op::test_bit(Alu8Data::Reg(C), 2), 2),
        0x52 => (Alu8Op::test_bit(Alu8Data::Reg(D), 2), 2),
        0x53 => (Alu8Op::test_bit(Alu8Data::Reg(E), 2), 2),
        0x54 => (Alu8Op::test_bit(Alu8Data::Reg(H), 2), 2),
        0x55 => (Alu8Op::test_bit(Alu8Data::Reg(L), 2), 2),
        0x56 => (Alu8Op::test_bit(Alu8Data::Addr(HL), 2), 4),
        0x57 => (Alu8Op::test_bit(Alu8Data::Reg(A), 2), 2),

        0x58 => (Alu8Op::test_bit(Alu8Data::Reg(B), 3), 2),
        0x59 => (Alu8Op::test_bit(Alu8Data::Reg(C), 3), 2),
        0x5A => (Alu8Op::test_bit(Alu8Data::Reg(D), 3), 2),
        0x5B => (Alu8Op::test_bit(Alu8Data::Reg(E), 3), 2),
        0x5C => (Alu8Op::test_bit(Alu8Data::Reg(H), 3), 2),
        0x5D => (Alu8Op::test_bit(Alu8Data::Reg(L), 3), 2),
        0x5E => (Alu8Op::test_bit(Alu8Data::Addr(HL), 3), 4),
        0x5F => (Alu8Op::test_bit(Alu8Data::Reg(A), 3), 2),

        0x60 => (Alu8Op::test_bit(Alu8Data::Reg(B), 4), 2),
        0x61 => (Alu8Op::test_bit(Alu8Data::Reg(C), 4), 2),
        0x62 => (Alu8Op::test_bit(Alu8Data::Reg(D), 4), 2),
        0x63 => (Alu8Op::test_bit(Alu8Data::Reg(E), 4), 2),
        0x64 => (Alu8Op::test_bit(Alu8Data::Reg(H), 4), 2),
        0x65 => (Alu8Op::test_bit(Alu8Data::Reg(L), 4), 2),
        0x66 => (Alu8Op::test_bit(Alu8Data::Addr(HL), 4), 4),
        0x67 => (Alu8Op::test_bit(Alu8Data::Reg(A), 4), 2),

        0x68 => (Alu8Op::test_bit(Alu8Data::Reg(B), 5), 2),
        0x69 => (Alu8Op::test_bit(Alu8Data::Reg(C), 5), 2),
        0x6A => (Alu8Op::test_bit(Alu8Data::Reg(D), 5), 2),
        0x6B => (Alu8Op::test_bit(Alu8Data::Reg(E), 5), 2),
        0x6C => (Alu8Op::test_bit(Alu8Data::Reg(H), 5), 2),
        0x6D => (Alu8Op::test_bit(Alu8Data::Reg(L), 5), 2),
        0x6E => (Alu8Op::test_bit(Alu8Data::Addr(HL), 5), 4),
        0x6F => (Alu8Op::test_bit(Alu8Data::Reg(A), 5), 2),

        0x70 => (Alu8Op::test_bit(Alu8Data::Reg(B), 6), 2),
        0x71 => (Alu8Op::test_bit(Alu8Data::Reg(C), 6), 2),
        0x72 => (Alu8Op::test_bit(Alu8Data::Reg(D), 6), 2),
        0x73 => (Alu8Op::test_bit(Alu8Data::Reg(E), 6), 2),
        0x74 => (Alu8Op::test_bit(Alu8Data::Reg(H), 6), 2),
        0x75 => (Alu8Op::test_bit(Alu8Data::Reg(L), 6), 2),
        0x76 => (Alu8Op::test_bit(Alu8Data::Addr(HL), 6), 4),
        0x77 => (Alu8Op::test_bit(Alu8Data::Reg(A), 6), 2),

        0x78 => (Alu8Op::test_bit(Alu8Data::Reg(B), 7), 2),
        0x79 => (Alu8Op::test_bit(Alu8Data::Reg(C), 7), 2),
        0x7A => (Alu8Op::test_bit(Alu8Data::Reg(D), 7), 2),
        0x7B => (Alu8Op::test_bit(Alu8Data::Reg(E), 7), 2),
        0x7C => (Alu8Op::test_bit(Alu8Data::Reg(H), 7), 2),
        0x7D => (Alu8Op::test_bit(Alu8Data::Reg(L), 7), 2),
        0x7E => (Alu8Op::test_bit(Alu8Data::Addr(HL), 7), 4),
        0x7F => (Alu8Op::test_bit(Alu8Data::Reg(A), 7), 2),

        // Reset Bits
        0x80 => (Alu8Op::reset_bit(Alu8Data::Reg(B), 0), 2),
        0x81 => (Alu8Op::reset_bit(Alu8Data::Reg(C), 0), 2),
        0x82 => (Alu8Op::reset_bit(Alu8Data::Reg(D), 0), 2),
        0x83 => (Alu8Op::reset_bit(Alu8Data::Reg(E), 0), 2),
        0x84 => (Alu8Op::reset_bit(Alu8Data::Reg(H), 0), 2),
        0x85 => (Alu8Op::reset_bit(Alu8Data::Reg(L), 0), 2),
        0x86 => (Alu8Op::reset_bit(Alu8Data::Addr(HL), 0), 4),
        0x87 => (Alu8Op::reset_bit(Alu8Data::Reg(A), 0), 2),

        0x88 => (Alu8Op::reset_bit(Alu8Data::Reg(B), 1), 2),
        0x89 => (Alu8Op::reset_bit(Alu8Data::Reg(C), 1), 2),
        0x8A => (Alu8Op::reset_bit(Alu8Data::Reg(D), 1), 2),
        0x8B => (Alu8Op::reset_bit(Alu8Data::Reg(E), 1), 2),
        0x8C => (Alu8Op::reset_bit(Alu8Data::Reg(H), 1), 2),
        0x8D => (Alu8Op::reset_bit(Alu8Data::Reg(L), 1), 2),
        0x8E => (Alu8Op::reset_bit(Alu8Data::Addr(HL), 1), 4),
        0x8F => (Alu8Op::reset_bit(Alu8Data::Reg(A), 1), 2),

        0x90 => (Alu8Op::reset_bit(Alu8Data::Reg(B), 2), 2),
        0x91 => (Alu8Op::reset_bit(Alu8Data::Reg(C), 2), 2),
        0x92 => (Alu8Op::reset_bit(Alu8Data::Reg(D), 2), 2),
        0x93 => (Alu8Op::reset_bit(Alu8Data::Reg(E), 2), 2),
        0x94 => (Alu8Op::reset_bit(Alu8Data::Reg(H), 2), 2),
        0x95 => (Alu8Op::reset_bit(Alu8Data::Reg(L), 2), 2),
        0x96 => (Alu8Op::reset_bit(Alu8Data::Addr(HL), 2), 4),
        0x97 => (Alu8Op::reset_bit(Alu8Data::Reg(A), 2), 2),

        0x98 => (Alu8Op::reset_bit(Alu8Data::Reg(B), 3), 2),
        0x99 => (Alu8Op::reset_bit(Alu8Data::Reg(C), 3), 2),
        0x9A => (Alu8Op::reset_bit(Alu8Data::Reg(D), 3), 2),
        0x9B => (Alu8Op::reset_bit(Alu8Data::Reg(E), 3), 2),
        0x9C => (Alu8Op::reset_bit(Alu8Data::Reg(H), 3), 2),
        0x9D => (Alu8Op::reset_bit(Alu8Data::Reg(L), 3), 2),
        0x9E => (Alu8Op::reset_bit(Alu8Data::Addr(HL), 3), 4),
        0x9F => (Alu8Op::reset_bit(Alu8Data::Reg(A), 3), 2),

        0xA0 => (Alu8Op::reset_bit(Alu8Data::Reg(B), 4), 2),
        0xA1 => (Alu8Op::reset_bit(Alu8Data::Reg(C), 4), 2),
        0xA2 => (Alu8Op::reset_bit(Alu8Data::Reg(D), 4), 2),
        0xA3 => (Alu8Op::reset_bit(Alu8Data::Reg(E), 4), 2),
        0xA4 => (Alu8Op::reset_bit(Alu8Data::Reg(H), 4), 2),
        0xA5 => (Alu8Op::reset_bit(Alu8Data::Reg(L), 4), 2),
        0xA6 => (Alu8Op::reset_bit(Alu8Data::Addr(HL), 4), 4),
        0xA7 => (Alu8Op::reset_bit(Alu8Data::Reg(A), 4), 2),

        0xA8 => (Alu8Op::reset_bit(Alu8Data::Reg(B), 5), 2),
        0xA9 => (Alu8Op::reset_bit(Alu8Data::Reg(C), 5), 2),
        0xAA => (Alu8Op::reset_bit(Alu8Data::Reg(D), 5), 2),
        0xAB => (Alu8Op::reset_bit(Alu8Data::Reg(E), 5), 2),
        0xAC => (Alu8Op::reset_bit(Alu8Data::Reg(H), 5), 2),
        0xAD => (Alu8Op::reset_bit(Alu8Data::Reg(L), 5), 2),
        0xAE => (Alu8Op::reset_bit(Alu8Data::Addr(HL), 5), 4),
        0xAF => (Alu8Op::reset_bit(Alu8Data::Reg(A), 5), 2),

        0xB0 => (Alu8Op::reset_bit(Alu8Data::Reg(B), 6), 2),
        0xB1 => (Alu8Op::reset_bit(Alu8Data::Reg(C), 6), 2),
        0xB2 => (Alu8Op::reset_bit(Alu8Data::Reg(D), 6), 2),
        0xB3 => (Alu8Op::reset_bit(Alu8Data::Reg(E), 6), 2),
        0xB4 => (Alu8Op::reset_bit(Alu8Data::Reg(H), 6), 2),
        0xB5 => (Alu8Op::reset_bit(Alu8Data::Reg(L), 6), 2),
        0xB6 => (Alu8Op::reset_bit(Alu8Data::Addr(HL), 6), 4),
        0xB7 => (Alu8Op::reset_bit(Alu8Data::Reg(A), 6), 2),

        0xB8 => (Alu8Op::reset_bit(Alu8Data::Reg(B), 7), 2),
        0xB9 => (Alu8Op::reset_bit(Alu8Data::Reg(C), 7), 2),
        0xBA => (Alu8Op::reset_bit(Alu8Data::Reg(D), 7), 2),
        0xBB => (Alu8Op::reset_bit(Alu8Data::Reg(E), 7), 2),
        0xBC => (Alu8Op::reset_bit(Alu8Data::Reg(H), 7), 2),
        0xBD => (Alu8Op::reset_bit(Alu8Data::Reg(L), 7), 2),
        0xBE => (Alu8Op::reset_bit(Alu8Data::Addr(HL), 7), 4),
        0xBF => (Alu8Op::reset_bit(Alu8Data::Reg(A), 7), 2),

        // Set bits.
        0xC0 => (Alu8Op::set_bit(Alu8Data::Reg(B), 0), 2),
        0xC1 => (Alu8Op::set_bit(Alu8Data::Reg(C), 0), 2),
        0xC2 => (Alu8Op::set_bit(Alu8Data::Reg(D), 0), 2),
        0xC3 => (Alu8Op::set_bit(Alu8Data::Reg(E), 0), 2),
        0xC4 => (Alu8Op::set_bit(Alu8Data::Reg(H), 0), 2),
        0xC5 => (Alu8Op::set_bit(Alu8Data::Reg(L), 0), 2),
        0xC6 => (Alu8Op::set_bit(Alu8Data::Addr(HL), 0), 4),
        0xC7 => (Alu8Op::set_bit(Alu8Data::Reg(A), 0), 2),

        0xC8 => (Alu8Op::set_bit(Alu8Data::Reg(B), 1), 2),
        0xC9 => (Alu8Op::set_bit(Alu8Data::Reg(C), 1), 2),
        0xCA => (Alu8Op::set_bit(Alu8Data::Reg(D), 1), 2),
        0xCB => (Alu8Op::set_bit(Alu8Data::Reg(E), 1), 2),
        0xCC => (Alu8Op::set_bit(Alu8Data::Reg(H), 1), 2),
        0xCD => (Alu8Op::set_bit(Alu8Data::Reg(L), 1), 2),
        0xCE => (Alu8Op::set_bit(Alu8Data::Addr(HL), 1), 4),
        0xCF => (Alu8Op::set_bit(Alu8Data::Reg(A), 1), 2),

        0xD0 => (Alu8Op::set_bit(Alu8Data::Reg(B), 2), 2),
        0xD1 => (Alu8Op::set_bit(Alu8Data::Reg(C), 2), 2),
        0xD2 => (Alu8Op::set_bit(Alu8Data::Reg(D), 2), 2),
        0xD3 => (Alu8Op::set_bit(Alu8Data::Reg(E), 2), 2),
        0xD4 => (Alu8Op::set_bit(Alu8Data::Reg(H), 2), 2),
        0xD5 => (Alu8Op::set_bit(Alu8Data::Reg(L), 2), 2),
        0xD6 => (Alu8Op::set_bit(Alu8Data::Addr(HL), 2), 4),
        0xD7 => (Alu8Op::set_bit(Alu8Data::Reg(A), 2), 2),

        0xD8 => (Alu8Op::set_bit(Alu8Data::Reg(B), 3), 2),
        0xD9 => (Alu8Op::set_bit(Alu8Data::Reg(C), 3), 2),
        0xDA => (Alu8Op::set_bit(Alu8Data::Reg(D), 3), 2),
        0xDB => (Alu8Op::set_bit(Alu8Data::Reg(E), 3), 2),
        0xDC => (Alu8Op::set_bit(Alu8Data::Reg(H), 3), 2),
        0xDD => (Alu8Op::set_bit(Alu8Data::Reg(L), 3), 2),
        0xDE => (Alu8Op::set_bit(Alu8Data::Addr(HL), 3), 4),
        0xDF => (Alu8Op::set_bit(Alu8Data::Reg(A), 3), 2),

        0xE0 => (Alu8Op::set_bit(Alu8Data::Reg(B), 4), 2),
        0xE1 => (Alu8Op::set_bit(Alu8Data::Reg(C), 4), 2),
        0xE2 => (Alu8Op::set_bit(Alu8Data::Reg(D), 4), 2),
        0xE3 => (Alu8Op::set_bit(Alu8Data::Reg(E), 4), 2),
        0xE4 => (Alu8Op::set_bit(Alu8Data::Reg(H), 4), 2),
        0xE5 => (Alu8Op::set_bit(Alu8Data::Reg(L), 4), 2),
        0xE6 => (Alu8Op::set_bit(Alu8Data::Addr(HL), 4), 4),
        0xE7 => (Alu8Op::set_bit(Alu8Data::Reg(A), 4), 2),

        0xE8 => (Alu8Op::set_bit(Alu8Data::Reg(B), 5), 2),
        0xE9 => (Alu8Op::set_bit(Alu8Data::Reg(C), 5), 2),
        0xEA => (Alu8Op::set_bit(Alu8Data::Reg(D), 5), 2),
        0xEB => (Alu8Op::set_bit(Alu8Data::Reg(E), 5), 2),
        0xEC => (Alu8Op::set_bit(Alu8Data::Reg(H), 5), 2),
        0xED => (Alu8Op::set_bit(Alu8Data::Reg(L), 5), 2),
        0xEE => (Alu8Op::set_bit(Alu8Data::Addr(HL), 5), 4),
        0xEF => (Alu8Op::set_bit(Alu8Data::Reg(A), 5), 2),

        0xF0 => (Alu8Op::set_bit(Alu8Data::Reg(B), 6), 2),
        0xF1 => (Alu8Op::set_bit(Alu8Data::Reg(C), 6), 2),
        0xF2 => (Alu8Op::set_bit(Alu8Data::Reg(D), 6), 2),
        0xF3 => (Alu8Op::set_bit(Alu8Data::Reg(E), 6), 2),
        0xF4 => (Alu8Op::set_bit(Alu8Data::Reg(H), 6), 2),
        0xF5 => (Alu8Op::set_bit(Alu8Data::Reg(L), 6), 2),
        0xF6 => (Alu8Op::set_bit(Alu8Data::Addr(HL), 6), 4),
        0xF7 => (Alu8Op::set_bit(Alu8Data::Reg(A), 6), 2),

        0xF8 => (Alu8Op::set_bit(Alu8Data::Reg(B), 7), 2),
        0xF9 => (Alu8Op::set_bit(Alu8Data::Reg(C), 7), 2),
        0xFA => (Alu8Op::set_bit(Alu8Data::Reg(D), 7), 2),
        0xFB => (Alu8Op::set_bit(Alu8Data::Reg(E), 7), 2),
        0xFC => (Alu8Op::set_bit(Alu8Data::Reg(H), 7), 2),
        0xFD => (Alu8Op::set_bit(Alu8Data::Reg(L), 7), 2),
        0xFE => (Alu8Op::set_bit(Alu8Data::Addr(HL), 7), 4),
        0xFF => (Alu8Op::set_bit(Alu8Data::Reg(A), 7), 2),

        // Needed to satisfy exhaustive checker, but completely unreachable.
        _ => panic!("Invalid opcode!"),
    };

    (Op::Alu8(alu_op), 2, time)
}
