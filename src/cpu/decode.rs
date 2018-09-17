use std::fmt;

use cpu;
use cpu::registers::{ShortReg, WideReg};

///! Op
/// TODO(slongfield): Encode the microops that make up these instructions, and the flags that
/// they affect. Right now, mostly just doing this to display the instructions.
pub enum Op {
    Nop,
    AluOp,
    Jump(Data),
    Move(ShortReg, ShortReg),
    Set(ShortReg, u8),
    SetWide(WideReg, u16),
    Load(ShortReg, Address),
    Store(Address, ShortReg),
    StoreAndIncrement(WideReg, ShortReg),
    StoreAndDecrement(WideReg, ShortReg),
    LoadAndIncrement(ShortReg, WideReg),
    LoadAndDecrement(ShortReg, WideReg),
    Unknown(u8),
}

pub enum AluOp {
    WideInc(WideReg),
    Inc(ShortReg),
    WideDec(WideReg),
    Dec(ShortReg),
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
    match rom[pc] {
        0x00 => (Op::Nop, 1),
        0x01 => {
            let val = cpu::bytes_to_u16(&rom[(pc + 1)..(pc + 3)]);
            (Op::SetWide(WideReg::BC, val), 3)
        }
        0x02 => (Op::Store(Address::Register16(WideReg::BC), ShortReg::A), 1),
        0x03 => (Op::AluOp(AluOp::WideInc(WideReg::BC)), 1),
        0x04 => (Op::AluOp(AluOp::Inc(ShortReg::B)), 1),
        0x05 => (Op::AluOp(AluOp::Dec(ShortReg::B)), 1),
        0x06 => (Op::Set(ShortReg::B, rom[pc+1]), 2),
        0x07 => (Op::RotateLeft(
                0xC3 => {
                    let dest = cpu::bytes_to_u16(&rom[(pc + 1)..(pc + 3)]);
                    (Op::Jump(Data::Immediate16(dest)), 3)
                }
                code => (Op::Unknown(code), 1),
                }
                }
