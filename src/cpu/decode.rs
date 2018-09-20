use std::fmt;

use cpu;
use cpu::registers::Reg16::{self, AF, BC, DE, HL, SP};
use cpu::registers::Reg8::{self, A, B, C, D, E, H, L};

///! Op
/// TODO(slongfield): Encode the microops that make up these instructions, and the flags that
/// they affect. Right now, mostly just doing this to display the instructions.
pub enum Op {
    Nop,
    AluOp(AluOp),
    Jump(Data),
    Move(Reg8, Reg8),
    Set(Reg8, u8),
    SetWide(Reg16, u16),
    Load(Reg8, Address),
    Store(Address, Reg8),
    WideStore(Address, Reg16),
    StoreAndIncrement(Address, Reg8),
    StoreAndDecrement(Address, Reg8),
    LoadAndIncrement(Reg8, Address),
    LoadAndDecrement(Reg8, Address),
    Unknown(u8),
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Op::Nop => write!(f, "NOP"),
            Op::AluOp(op) => write!(f, "{}", op),
            Op::Jump(dest) => write!(f, "JP {}", dest),
            Op::Move(src, dest) => write!(f, "LD {:?} {:?}", src, dest),
            Op::Set(dest, val) => write!(f, "LD {:?} 0x{:x}", dest, val),
            Op::SetWide(dest, val) => write!(f, "LD {:?} 0x{:x}", dest, val),
            Op::Load(dest, addr) => write!(f, "LD {:?} ({:?})", dest, addr),
            Op::Store(addr, src) => write!(f, "LD ({:?}) {:?}", addr, src),
            Op::StoreAndIncrement(addr, src) => write!(f, "LD ({:?}+) {:?}", addr, src),
            Op::StoreAndDecrement(addr, src) => write!(f, "LD ({:?}-) {:?}", addr, src),
            Op::LoadAndIncrement(dest, addr) => write!(f, "LD {:?} ({:?}+)", dest, addr),
            Op::LoadAndDecrement(dest, addr) => write!(f, "LD {:?} ({:?}-)", dest, addr),
            Op::Unknown(code) => write!(f, "Don't know how to display: 0x{:x}", code),
            _ => write!(f, "Missed case!"),
        }
    }
}

pub enum AluOp {
    // Accumulator register has special rotate instructions that run faster.
    Add(Data), // Add to accumulator.
    AddWithCarry(Data),
    And(Data),     // And with accumulator.
    Compare(Data), // Compare with accumulator.
    Dec(Reg8),
    Inc(Reg8),
    Or(Data), // Or with accumulator.
    RotateLeftIntoCarry,
    RotateLeftThroughCarry,
    RotateRegLeftIntoCarry(Reg8),
    RotateRegLeftThroughCarry(Reg8),
    RotateRegRightIntoCarry(Reg8),
    RotateRegRightThroughCarry(Reg8),
    RotateRightIntoCarry,
    RotateRightThroughCarry,
    RotateReg16LeftIntoCarry(Reg16),
    RotateReg16LeftThroughCarry(Reg16),
    RotateReg16RightIntoCarry(Reg16),
    RotateReg16RightThroughCarry(Reg16),
    Sub(Data), // Subtract from accumulator.
    SubWithCarry(Data),
    WideAdd(Reg16, Reg16),
    WideDec(Reg16),
    WideInc(Reg16),
    Xor(Data), // Xor with accumulator.
    Unknown,
}

impl fmt::Display for AluOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AluOp::Add(data) => write!(f, "ADD A {:?}", data),
            AluOp::AddWithCarry(data) => write!(f, "ADC {:?}", data),
            AluOp::And(data) => write!(f, "AND {:?}", data),
            AluOp::Compare(data) => write!(f, "CMP {:?}", data),
            AluOp::Dec(reg) => write!(f, "DEC {:?}", reg),
            AluOp::Inc(reg) => write!(f, "INC {:?}", reg),
            AluOp::Or(data) => write!(f, "OR {:?}", data),
            AluOp::RotateLeftIntoCarry => write!(f, "RLCA"),
            AluOp::RotateLeftThroughCarry => write!(f, "RLC"),
            AluOp::RotateRegLeftIntoCarry(reg) => write!(f, "RLC {:?}", reg),
            AluOp::RotateRegLeftThroughCarry(reg) => write!(f, "RL {:?}", reg),
            AluOp::RotateRegRightIntoCarry(reg) => write!(f, "RRC {:?}", reg),
            AluOp::RotateRegRightThroughCarry(reg) => write!(f, "RR {:?}", reg),
            AluOp::RotateRightIntoCarry => write!(f, "RRCA"),
            AluOp::RotateRightThroughCarry => write!(f, "RRA"),
            AluOp::RotateReg16LeftIntoCarry(reg) => write!(f, "RLC {:?}", reg),
            AluOp::RotateReg16LeftThroughCarry(reg) => write!(f, "RL {:?}", reg),
            AluOp::RotateReg16RightIntoCarry(reg) => write!(f, "RRC {:?}", reg),
            AluOp::RotateReg16RightThroughCarry(reg) => write!(f, "RR {:?}", reg),
            AluOp::Sub(data) => write!(f, "SUB {:?}", data),
            AluOp::SubWithCarry(data) => write!(f, "SBC {:?}", data),
            AluOp::WideAdd(reg_x, reg_y) => write!(f, "ADD {:?} {:?}", reg_x, reg_y),
            AluOp::WideDec(reg) => write!(f, "DEC {:?}", reg),
            AluOp::WideInc(reg) => write!(f, "INC {:?}", reg),
            AluOp::Xor(data) => write!(f, "XOR {:?}", data),
            AluOp::Unknown => write!(f, "Unknown ALU OP!!"),
        }
    }
}

///! Data for use in ops.
#[derive(Debug)]
pub enum Data {
    Register8(Reg8),
    Register16(Reg16),
    Immediate8(u8),
    Immediate16(u16),
}

#[derive(Debug)]
pub enum Address {
    Register16(Reg16),
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
///! bytes in that op, and the number of cycles it runs for.
pub fn decode(rom: &[u8], pc: usize) -> (Op, usize, usize) {
    if let Some((op, size, time)) = decode_alu(&rom, pc) {
        return (op, size, time);
    }
    if let Some((op, size, time)) = decode_load(&rom, pc) {
        return (op, size, time);
    }
    match rom[pc] {
        0x00 => (Op::Nop, 1, 1),
        0xC3 => {
            let dest = cpu::bytes_to_u16(&rom[(pc + 1)..(pc + 3)]);
            (Op::Jump(Data::Immediate16(dest)), 3, 3)
        }
        code => (Op::Unknown(code), 1, 0),
    }
}

///! Decode ALU operations.
fn decode_alu(rom: &[u8], pc: usize) -> Option<(Op, usize, usize)> {
    let inst = match rom[pc] {
        0x03 => (AluOp::WideInc(BC), 1, 1),

        0x04 => (AluOp::Inc(B), 1, 1),
        0x14 => (AluOp::Inc(D), 1, 1),
        0x24 => (AluOp::Inc(H), 1, 1),
        0x34 => (AluOp::WideInc(HL), 1, 1),

        0x05 => (AluOp::Dec(B), 1, 1),
        0x15 => (AluOp::Dec(D), 1, 1),
        0x25 => (AluOp::Dec(H), 1, 1),
        0x35 => (AluOp::WideDec(HL), 1, 1),

        0x07 => (AluOp::RotateLeftIntoCarry, 1, 1),
        0x17 => (AluOp::RotateLeftThroughCarry, 1, 1),
        0x0F => (AluOp::RotateRightIntoCarry, 1, 1),
        0x1F => (AluOp::RotateRightThroughCarry, 1, 1),

        // TODO(slongifeld) 0x27, 0x37, 0x2f, 0x3F
        0x09 => (AluOp::WideAdd(HL, BC), 1, 2),
        0x19 => (AluOp::WideAdd(HL, DE), 1, 2),
        0x29 => (AluOp::WideAdd(HL, HL), 1, 2),
        0x39 => (AluOp::WideAdd(HL, SP), 1, 2),

        0x0B => (AluOp::WideDec(BC), 1, 2),
        0x1B => (AluOp::WideDec(DE), 1, 2),
        0x2B => (AluOp::WideDec(HL), 1, 2),
        0x3B => (AluOp::WideDec(SP), 1, 2),

        0x0C => (AluOp::Inc(C), 1, 1),
        0x1C => (AluOp::Inc(D), 1, 1),
        0x2C => (AluOp::Inc(L), 1, 1),
        0x3C => (AluOp::Inc(A), 1, 1),

        0x0D => (AluOp::Dec(D), 1, 1),
        0x1D => (AluOp::Dec(E), 1, 1),
        0x2D => (AluOp::Dec(L), 1, 1),
        0x3D => (AluOp::Dec(A), 1, 1),

        0x80 => (AluOp::Add(Data::Register8(B)), 1, 1),
        0x81 => (AluOp::Add(Data::Register8(C)), 1, 1),
        0x82 => (AluOp::Add(Data::Register8(D)), 1, 1),
        0x83 => (AluOp::Add(Data::Register8(E)), 1, 1),
        0x84 => (AluOp::Add(Data::Register8(H)), 1, 1),
        0x85 => (AluOp::Add(Data::Register8(L)), 1, 1),
        0x86 => (AluOp::Add(Data::Register16(HL)), 1, 1),
        0x87 => (AluOp::Add(Data::Register8(A)), 1, 2),
        0x88 => (AluOp::AddWithCarry(Data::Register8(B)), 1, 1),
        0x89 => (AluOp::AddWithCarry(Data::Register8(C)), 1, 1),
        0x8A => (AluOp::AddWithCarry(Data::Register8(D)), 1, 1),
        0x8B => (AluOp::AddWithCarry(Data::Register8(E)), 1, 1),
        0x8C => (AluOp::AddWithCarry(Data::Register8(H)), 1, 1),
        0x8D => (AluOp::AddWithCarry(Data::Register8(L)), 1, 1),
        0x8E => (AluOp::AddWithCarry(Data::Register16(HL)), 1, 2),
        0x8F => (AluOp::AddWithCarry(Data::Register8(A)), 1, 1),

        0x90 => (AluOp::Sub(Data::Register8(B)), 1, 1),
        0x91 => (AluOp::Sub(Data::Register8(C)), 1, 1),
        0x92 => (AluOp::Sub(Data::Register8(D)), 1, 1),
        0x93 => (AluOp::Sub(Data::Register8(E)), 1, 1),
        0x94 => (AluOp::Sub(Data::Register8(H)), 1, 1),
        0x95 => (AluOp::Sub(Data::Register8(L)), 1, 1),
        0x96 => (AluOp::Sub(Data::Register16(HL)), 1, 2),
        0x97 => (AluOp::Sub(Data::Register8(A)), 1, 1),
        0x98 => (AluOp::SubWithCarry(Data::Register8(B)), 1, 1),
        0x99 => (AluOp::SubWithCarry(Data::Register8(C)), 1, 1),
        0x9A => (AluOp::SubWithCarry(Data::Register8(D)), 1, 1),
        0x9B => (AluOp::SubWithCarry(Data::Register8(E)), 1, 1),
        0x9C => (AluOp::SubWithCarry(Data::Register8(H)), 1, 1),
        0x9D => (AluOp::SubWithCarry(Data::Register8(L)), 1, 1),
        0x9E => (AluOp::SubWithCarry(Data::Register16(HL)), 1, 2),
        0x9F => (AluOp::SubWithCarry(Data::Register8(A)), 1, 1),

        0xA0 => (AluOp::And(Data::Register8(B)), 1, 1),
        0xA1 => (AluOp::And(Data::Register8(C)), 1, 1),
        0xA2 => (AluOp::And(Data::Register8(D)), 1, 1),
        0xA3 => (AluOp::And(Data::Register8(E)), 1, 1),
        0xA4 => (AluOp::And(Data::Register8(H)), 1, 1),
        0xA5 => (AluOp::And(Data::Register8(L)), 1, 1),
        0xA6 => (AluOp::And(Data::Register16(HL)), 1, 2),
        0xA7 => (AluOp::And(Data::Register8(A)), 1, 1),

        0xA8 => (AluOp::Xor(Data::Register8(B)), 1, 1),
        0xA9 => (AluOp::Xor(Data::Register8(C)), 1, 1),
        0xAA => (AluOp::Xor(Data::Register8(D)), 1, 1),
        0xAB => (AluOp::Xor(Data::Register8(E)), 1, 1),
        0xAC => (AluOp::Xor(Data::Register8(H)), 1, 1),
        0xAD => (AluOp::Xor(Data::Register8(L)), 1, 1),
        0xAE => (AluOp::Xor(Data::Register16(HL)), 1, 2),
        0xAF => (AluOp::Xor(Data::Register8(A)), 1, 1),

        0xB0 => (AluOp::Or(Data::Register8(B)), 1, 1),
        0xB1 => (AluOp::Or(Data::Register8(C)), 1, 1),
        0xB2 => (AluOp::Or(Data::Register8(D)), 1, 1),
        0xB3 => (AluOp::Or(Data::Register8(E)), 1, 1),
        0xB4 => (AluOp::Or(Data::Register8(H)), 1, 1),
        0xB5 => (AluOp::Or(Data::Register8(L)), 1, 1),
        0xB6 => (AluOp::Or(Data::Register16(HL)), 1, 2),
        0xB7 => (AluOp::Or(Data::Register8(A)), 1, 1),

        0xB8 => (AluOp::Compare(Data::Register8(B)), 1, 1),
        0xB9 => (AluOp::Compare(Data::Register8(C)), 1, 1),
        0xBA => (AluOp::Compare(Data::Register8(D)), 1, 1),
        0xBB => (AluOp::Compare(Data::Register8(E)), 1, 1),
        0xBC => (AluOp::Compare(Data::Register8(H)), 1, 1),
        0xBD => (AluOp::Compare(Data::Register8(L)), 1, 1),
        0xBE => (AluOp::Compare(Data::Register16(HL)), 1, 2),
        0xBF => (AluOp::Compare(Data::Register8(A)), 1, 1),

        0xC6 => (AluOp::Add(Data::Immediate8(rom[(pc + 1)])), 1, 2),
        0xD6 => (AluOp::Sub(Data::Immediate8(rom[(pc + 1)])), 1, 2),
        0xE6 => (AluOp::And(Data::Immediate8(rom[(pc + 1)])), 1, 2),
        0xF6 => (AluOp::Or(Data::Immediate8(rom[(pc + 1)])), 1, 2),
        0xCE => (AluOp::AddWithCarry(Data::Immediate8(rom[(pc + 1)])), 1, 2),
        0xDE => (AluOp::SubWithCarry(Data::Immediate8(rom[(pc + 1)])), 1, 2),
        0xEE => (AluOp::Xor(Data::Immediate8(rom[(pc + 1)])), 1, 2),
        0xFE => (AluOp::Compare(Data::Immediate8(rom[(pc + 1)])), 1, 2),

        _ => (AluOp::Unknown, 0, 0),
    };
    match inst {
        (AluOp::Unknown, _, _) => None,
        (op, size, time) => Some((Op::AluOp(op), size, time)),
    }
}

///! Decode move, load, and store operations.
fn decode_load(rom: &[u8], pc: usize) -> Option<(Op, usize, usize)> {
    let inst = match rom[pc] {
        0x01 => (
            Op::SetWide(BC, cpu::bytes_to_u16(&rom[(pc + 1)..(pc + 3)])),
            3,
            3,
        ),
        0x11 => (
            Op::SetWide(BC, cpu::bytes_to_u16(&rom[(pc + 1)..(pc + 3)])),
            3,
            3,
        ),
        0x21 => (
            Op::SetWide(BC, cpu::bytes_to_u16(&rom[(pc + 1)..(pc + 3)])),
            3,
            3,
        ),
        0x31 => (
            Op::SetWide(BC, cpu::bytes_to_u16(&rom[(pc + 1)..(pc + 3)])),
            3,
            3,
        ),

        0x02 => (Op::Store(Address::Register16(BC), A), 1, 2),
        0x12 => (Op::Store(Address::Register16(DE), A), 1, 2),
        0x22 => (Op::StoreAndIncrement(Address::Register16(HL), A), 1, 2),
        0x32 => (Op::StoreAndDecrement(Address::Register16(HL), A), 1, 2),

        0x06 => (Op::Set(B, rom[pc + 1]), 2, 2),
        0x16 => (Op::Set(D, rom[pc + 1]), 2, 2),
        0x26 => (Op::Set(H, rom[pc + 1]), 2, 2),
        0x36 => (Op::SetWide(HL, u16::from(rom[pc + 1])), 2, 2),

        0x08 => {
            let addr = cpu::bytes_to_u16(&rom[(pc + 1)..(pc + 3)]);
            (Op::WideStore(Address::Immediate16(addr), SP), 3, 5)
        }

        0x0E => (Op::Set(C, rom[pc + 1]), 2, 2),
        0x1E => (Op::Set(E, rom[pc + 1]), 2, 2),
        0x2E => (Op::Set(L, rom[pc + 1]), 2, 2),
        0x3E => (Op::Set(A, rom[pc + 1]), 2, 2),

        0x0A => (Op::Load(A, Address::Register16(BC)), 1, 2),
        0x1A => (Op::Load(A, Address::Register16(DE)), 1, 2),
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
        0x7E => (Op::Load(L, Address::Register16(HL)), 1, 2),

        0x70 => (Op::Store(Address::Register16(HL), B), 1, 2),
        0x71 => (Op::Store(Address::Register16(HL), C), 1, 2),
        0x72 => (Op::Store(Address::Register16(HL), D), 1, 2),
        0x73 => (Op::Store(Address::Register16(HL), E), 1, 2),
        0x74 => (Op::Store(Address::Register16(HL), H), 1, 2),
        0x75 => (Op::Store(Address::Register16(HL), L), 1, 2),
        0x77 => (Op::Store(Address::Register16(HL), A), 1, 2),

        code => (Op::Unknown(code), 0, 0),
    };
    match inst {
        (Op::Unknown(_), _, _) => None,
        (op, size, time) => Some((op, size, time)),
    }
}
