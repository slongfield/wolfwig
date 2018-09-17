use std::fmt;

use cpu;
use cpu::registers::ShortReg::{self, A, B, C, D, E, H, L};
use cpu::registers::WideReg::{self, AF, BC, DE, HL, PC, SP};

///! Op
/// TODO(slongfield): Encode the microops that make up these instructions, and the flags that
/// they affect. Right now, mostly just doing this to display the instructions.
pub enum Op {
    Nop,
    AluOp(AluOp),
    Jump(Data),
    Move(ShortReg, ShortReg),
    Set(ShortReg, u8),
    SetWide(WideReg, u16),
    Load(ShortReg, Address),
    Store(Address, ShortReg),
    WideStore(Address, WideReg),
    StoreAndIncrement(WideReg, ShortReg),
    StoreAndDecrement(WideReg, ShortReg),
    LoadAndIncrement(ShortReg, WideReg),
    LoadAndDecrement(ShortReg, WideReg),
    Unknown(u8),
}

pub enum AluOp {
    Add(Data), // Add to accumulator.
    Sub(Data), // Subtract from accumulator.
    WideAdd(WideReg, WideReg),
    WideInc(WideReg),
    Inc(ShortReg),
    WideDec(WideReg),
    Dec(ShortReg),
    // Accumulator register has special rotate instructions that run faster.
    RotateLeftIntoCarry,
    RotateLeftThroughCarry,
    RotateRightThroughCarry,
    RotateRightIntoCarry,
    RotateRegLeftIntoCarry(ShortReg),
    RotateRegLeftThroughCarry(ShortReg),
    RotateRegRightThroughCarry(ShortReg),
    RotateRegRightIntoCarry(ShortReg),
    RotateWideRegLeftIntoCarry(WideReg),
    RotateWideRegLeftThroughCarry(WideReg),
    RotateWideRegRightThroughCarry(WideReg),
    RotateWideRegRightIntoCarry(WideReg),
    Unknown,
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Op::Nop => write!(f, "NOP"),
            Op::Jump(dest) => write!(f, "JP {}", dest),
            Op::Unknown(code) => write!(f, "Don't know how to display: 0x{:x}", code),
            _ => write!(f, "Missed case!"),
        }
    }
}

///! Data for use in ops.
pub enum Data {
    Register8(ShortReg),
    Register16(WideReg),
    Immediate8(u8),
    Immediate16(u16),
}

pub enum Address {
    Register16(WideReg),
    Immediate16(u16),
    IoImmedite(u8),
    IoRegister, // Always register C.
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Data::Immediate16(val) => write!(f, "0x{:x}", val),
            _ => write!(f, "Unknown data type"),
        }
    }
}

///! Decode takes the ROM and current PC, and returns the Op a that PC, as well as the number of
///! bytes in that op.
pub fn decode(rom: &Vec<u8>, pc: usize) -> (Op, usize) {
    match decode_alu(&rom, pc) {
        Some((Op, usize)) => return (Op, usize),
        None => (),
    }
    match decode_load(&rom, pc) {
        Some((Op, usize)) => return (Op, usize),
        None => (),
    }
    match rom[pc] {
        0x00 => (Op::Nop, 1),
        0x01 => {
            let val = cpu::bytes_to_u16(&rom[(pc + 1)..(pc + 3)]);
            (Op::SetWide(BC, val), 3)
        }
        0x02 => (Op::Store(Address::Register16(BC), A), 1),
        0x06 => (Op::Set(B, rom[pc + 1]), 2),
        0x08 => {
            let addr = cpu::bytes_to_u16(&rom[(pc + 1)..(pc + 3)]);
            (Op::WideStore(Address::Immediate16(addr), SP), 3)
        }
        0x0A => (Op::Load(A, Address::Register16(BC)), 1),
        0xC3 => {
            let dest = cpu::bytes_to_u16(&rom[(pc + 1)..(pc + 3)]);
            (Op::Jump(Data::Immediate16(dest)), 3)
        }
        code => (Op::Unknown(code), 1),
    }
}

///! Decode ALU operations.
fn decode_alu(rom: &Vec<u8>, pc: usize) -> Option<(Op, usize)> {
    let inst = match rom[pc] {
        0x03 => (AluOp::WideInc(BC), 1),

        0x04 => (AluOp::Inc(B), 1),
        0x14 => (AluOp::Inc(D), 1),
        0x24 => (AluOp::Inc(H), 1),
        0x34 => (AluOp::WideInc(HL), 1),

        0x05 => (AluOp::Dec(B), 1),
        0x15 => (AluOp::Dec(D), 1),
        0x25 => (AluOp::Dec(H), 1),
        0x35 => (AluOp::WideDec(HL), 1),

        0x07 => (AluOp::RotateLeftIntoCarry, 1),
        0x17 => (AluOp::RotateLeftThroughCarry, 1),

        0x09 => (AluOp::WideAdd(HL, BC), 1),
        0x0B => (AluOp::WideDec(BC), 1),
        0x0C => (AluOp::Inc(C), 1),
        0x0D => (AluOp::Dec(D), 1),
        _ => (AluOp::Unknown, 0),
    };
    match inst {
        (AluOp::Unknown, _) => None,
        (op, size) => Some((Op::AluOp(op), size)),
    }
}

///! Decode move, load, and store operations.
fn decode_load(rom: &Vec<u8>, pc: usize) -> Option<(Op, usize)> {
    None
}
