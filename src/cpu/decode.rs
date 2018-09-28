use mem::model::Memory;
use std::fmt;

use cpu::registers::Flag::{self, Carry, NotCarry, NotZero, Zero};
use cpu::registers::Reg16::{self, AF, BC, DE, HL, SP};
use cpu::registers::Reg8::{self, A, B, C, D, E, H, L};
use util;

///! Op
/// TODO(slongfield): Encode the microops that make up these instructions, and the flags that
/// they affect. Right now, mostly just doing this to display the instructions.
#[derive(Debug)]
pub enum Op {
    AluOp(AluOp),
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
            Op::AluOp(op) => write!(f, "{}", op),
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
            Op::Stop => write!(f, "STOP"),
            Op::Store(addr, src) => write!(f, "LD ({}) {}", addr, src),
            Op::StoreAndDecrement(addr, src) => write!(f, "LD ({}-) {}", addr, src),
            Op::StoreAndIncrement(addr, src) => write!(f, "LD ({}+) {}", addr, src),
            Op::Unknown(code) => write!(f, "Don't know how to display: 0x{:X}", code),
            _ => write!(f, "Missed case!"),
        }
    }
}

// TODO(slongfield): Rework this structure.
// Op, Xsrc, YSrc, Dest
// All of the '16' ones appear to be incorrect. These actually modify a value at the memory
// location pointed to by the associated register, and the ALU is fully 8-bit.
#[derive(Debug)]
pub enum AluOp {
    // Accumulator register has special rotate instructions that run faster.
    Add(Data), // Add to accumulator.
    AddWithCarry(Data),
    And(Data), // And with accumulator.
    ClearCarry,
    Compare(Data), // Compare with accumulator.
    Complment,
    Dec(Reg8),
    DecimalAdjust,
    Inc(Reg8),
    Or(Data), // Or with accumulator.
    ResetBit(Reg8, u8),
    ResetBit16(Reg16, u8),
    Rotate16LeftIntoCarry(Reg16),
    Rotate16LeftThroughCarry(Reg16),
    Rotate16RightIntoCarry(Reg16),
    Rotate16RightThroughCarry(Reg16),
    Rotate8LeftIntoCarry(Reg8),
    Rotate8LeftThroughCarry(Reg8),
    Rotate8RightIntoCarry(Reg8),
    Rotate8RightThroughCarry(Reg8),
    RotateLeftIntoCarry,
    RotateLeftThroughCarry,
    RotateRightIntoCarry,
    RotateRightThroughCarry,
    SetBit(Reg8, u8),
    SetBit16(Reg16, u8),
    SetCarry,
    Shift16LeftArithmetic(Reg16),
    Shift16RightArithmetic(Reg16),
    Shift16RightLogical(Reg16),
    Shift8LeftArithmetic(Reg8),
    Shift8RightArithmetic(Reg8),
    Shift8RightLogical(Reg8),
    Sub(Data), // Subtract from accumulator.
    SubWithCarry(Data),
    Swap(Reg8),
    Swap16(Reg16),
    TestBit(Reg8, u8),
    TestBit16(Reg16, u8),
    Unknown,
    WideAdd(Reg16, Reg16),
    WideDec(Reg16),
    AddrDec(Reg16),
    WideInc(Reg16),
    AddrInc(Reg16),
    Xor(Data), // Xor with accumulator.
}

impl fmt::Display for AluOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AluOp::Add(data) => write!(f, "ADD A {}", data),
            AluOp::AddWithCarry(data) => write!(f, "ADC {}", data),
            AluOp::And(data) => write!(f, "AND {}", data),
            AluOp::ClearCarry => write!(f, "CCF"),
            AluOp::Compare(data) => write!(f, "CMP {}", data),
            AluOp::Complment => write!(f, "CPL"),
            AluOp::Dec(reg) => write!(f, "DEC {}", reg),
            AluOp::DecimalAdjust => write!(f, "DAA"),
            AluOp::Inc(reg) => write!(f, "INC {}", reg),
            AluOp::Or(data) => write!(f, "OR {}", data),
            AluOp::ResetBit(reg, bit) => write!(f, "RES {},{}", bit, reg),
            AluOp::ResetBit16(reg, bit) => write!(f, "RES {},({})", bit, reg),
            AluOp::Rotate16LeftIntoCarry(reg) => write!(f, "RLC ({})", reg),
            AluOp::Rotate16LeftThroughCarry(reg) => write!(f, "RL ({})", reg),
            AluOp::Rotate16RightIntoCarry(reg) => write!(f, "RRC ({})", reg),
            AluOp::Rotate16RightThroughCarry(reg) => write!(f, "RR ({})", reg),
            AluOp::Rotate8LeftIntoCarry(reg) => write!(f, "RLC {}", reg),
            AluOp::Rotate8LeftThroughCarry(reg) => write!(f, "RL {}", reg),
            AluOp::Rotate8RightIntoCarry(reg) => write!(f, "RRC {}", reg),
            AluOp::Rotate8RightThroughCarry(reg) => write!(f, "RR {}", reg),
            AluOp::RotateLeftIntoCarry => write!(f, "RLCA"),
            AluOp::RotateLeftThroughCarry => write!(f, "RLC"),
            AluOp::RotateRightIntoCarry => write!(f, "RRCA"),
            AluOp::RotateRightThroughCarry => write!(f, "RRA"),
            AluOp::SetBit(reg, bit) => write!(f, "SET {},{}", bit, reg),
            AluOp::SetBit16(reg, bit) => write!(f, "SET {},({})", bit, reg),
            AluOp::SetCarry => write!(f, "SCF"),
            AluOp::Shift16LeftArithmetic(reg) => write!(f, "SLA ({})", reg),
            AluOp::Shift16RightArithmetic(reg) => write!(f, "SRA ({})", reg),
            AluOp::Shift16RightLogical(reg) => write!(f, "SRL ({})", reg),
            AluOp::Shift8LeftArithmetic(reg) => write!(f, "SLA {}", reg),
            AluOp::Shift8RightArithmetic(reg) => write!(f, "SRA {}", reg),
            AluOp::Shift8RightLogical(reg) => write!(f, "SRL {}", reg),
            AluOp::Sub(data) => write!(f, "SUB {}", data),
            AluOp::SubWithCarry(data) => write!(f, "SBC {}", data),
            AluOp::Swap(reg) => write!(f, "SWAP {}", reg),
            AluOp::Swap16(reg) => write!(f, "SWAP ({})", reg),
            AluOp::TestBit(reg, bit) => write!(f, "BIT {},{}", bit, reg),
            AluOp::TestBit16(reg, bit) => write!(f, "BIT {},({})", bit, reg),
            AluOp::Unknown => write!(f, "Unknown ALU OP!!"),
            AluOp::WideAdd(reg_x, reg_y) => write!(f, "ADD {} {}", reg_x, reg_y),
            AluOp::WideDec(reg) => write!(f, "DEC {}", reg),
            AluOp::AddrDec(reg) => write!(f, "DEC ({})", reg),
            AluOp::WideInc(reg) => write!(f, "INC {}", reg),
            AluOp::AddrInc(reg) => write!(f, "INC ({})", reg),
            AluOp::Xor(data) => write!(f, "XOR {}", data),
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

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Data::Register8(reg) => write!(f, "{}", reg),
            Data::Register16(reg) => write!(f, "{}", reg),
            Data::Immediate8(data) => write!(f, "0x{:X}", data),
            Data::Immediate16(data) => write!(f, "0x{:X}", data),
        }
    }
}

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

///! Decode takes the ROM and current PC, and returns the Op a that PC, as well as the number of
///! bytes in that op, and the number of cycles it runs for.
pub fn decode(rom: &Memory, pc: usize) -> (Op, usize, usize) {
    if let Some((op, size, time)) = decode_alu(&rom, pc) {
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
        0x01 => (Op::Stop, 2, 1),
        0x76 => (Op::Halt, 1, 1),
        0xF3 => (Op::DisableInterrupts, 1, 1),
        0xFB => (Op::EnableInterrupts, 1, 1),
        0xCB => decode_extended(rom.read(pc + 1)),
        code => (Op::Unknown(code), 1, 0),
    }
}

///! Decode ALU operations.
fn decode_alu(rom: &Memory, pc: usize) -> Option<(Op, usize, usize)> {
    let imm8 = rom.read(pc + 1);
    let inst = match rom.read(pc) {
        0x03 => (AluOp::WideInc(BC), 1, 1),
        0x13 => (AluOp::WideInc(DE), 1, 1),
        0x23 => (AluOp::WideInc(HL), 1, 1),
        0x33 => (AluOp::WideInc(SP), 1, 1),

        0x04 => (AluOp::Inc(B), 1, 1),
        0x14 => (AluOp::Inc(D), 1, 1),
        0x24 => (AluOp::Inc(H), 1, 1),
        0x34 => (AluOp::AddrInc(HL), 1, 1),

        0x05 => (AluOp::Dec(B), 1, 1),
        0x15 => (AluOp::Dec(D), 1, 1),
        0x25 => (AluOp::Dec(H), 1, 1),
        0x35 => (AluOp::AddrDec(HL), 1, 1),

        0x07 => (AluOp::RotateLeftIntoCarry, 1, 1),
        0x17 => (AluOp::RotateLeftThroughCarry, 1, 1),
        0x0F => (AluOp::RotateRightIntoCarry, 1, 1),
        0x1F => (AluOp::RotateRightThroughCarry, 1, 1),

        0x27 => (AluOp::DecimalAdjust, 1, 1),
        0x2F => (AluOp::Complment, 1, 1),

        0x37 => (AluOp::SetCarry, 1, 1),
        0x3F => (AluOp::ClearCarry, 1, 1),

        0x09 => (AluOp::WideAdd(HL, BC), 1, 2),
        0x19 => (AluOp::WideAdd(HL, DE), 1, 2),
        0x29 => (AluOp::WideAdd(HL, HL), 1, 2),
        0x39 => (AluOp::WideAdd(HL, SP), 1, 2),

        0x0B => (AluOp::WideDec(BC), 1, 2),
        0x1B => (AluOp::WideDec(DE), 1, 2),
        0x2B => (AluOp::WideDec(HL), 1, 2),
        0x3B => (AluOp::WideDec(SP), 1, 2),

        0x0C => (AluOp::Inc(C), 1, 1),
        0x1C => (AluOp::Inc(E), 1, 1),
        0x2C => (AluOp::Inc(L), 1, 1),
        0x3C => (AluOp::Inc(A), 1, 1),

        0x0D => (AluOp::Dec(C), 1, 1),
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

        0xC6 => (AluOp::Add(Data::Immediate8(imm8)), 2, 1),
        0xD6 => (AluOp::Sub(Data::Immediate8(imm8)), 2, 1),
        0xE6 => (AluOp::And(Data::Immediate8(imm8)), 2, 1),
        0xF6 => (AluOp::Or(Data::Immediate8(imm8)), 2, 1),
        0xCE => (AluOp::AddWithCarry(Data::Immediate8(imm8)), 2, 1),
        0xDE => (AluOp::SubWithCarry(Data::Immediate8(imm8)), 2, 1),
        0xEE => (AluOp::Xor(Data::Immediate8(imm8)), 2, 1),
        0xFE => (AluOp::Compare(Data::Immediate8(imm8)), 2, 1),

        _ => (AluOp::Unknown, 0, 0),
    };
    match inst {
        (AluOp::Unknown, _, _) => None,
        (op, size, time) => Some((Op::AluOp(op), size, time)),
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
        0x36 => (Op::SetWide(HL, u16::from(imm8)), 2, 2),

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
        0x00 => (AluOp::Rotate8LeftIntoCarry(B), 2),
        0x01 => (AluOp::Rotate8LeftIntoCarry(C), 2),
        0x02 => (AluOp::Rotate8LeftIntoCarry(D), 2),
        0x03 => (AluOp::Rotate8LeftIntoCarry(E), 2),
        0x04 => (AluOp::Rotate8LeftIntoCarry(H), 2),
        0x05 => (AluOp::Rotate8LeftIntoCarry(L), 2),
        0x06 => (AluOp::Rotate16LeftIntoCarry(HL), 4),
        0x07 => (AluOp::Rotate8LeftIntoCarry(A), 2),

        0x08 => (AluOp::Rotate8RightIntoCarry(B), 2),
        0x09 => (AluOp::Rotate8RightIntoCarry(C), 2),
        0x0A => (AluOp::Rotate8RightIntoCarry(D), 2),
        0x0B => (AluOp::Rotate8RightIntoCarry(E), 2),
        0x0C => (AluOp::Rotate8RightIntoCarry(H), 2),
        0x0D => (AluOp::Rotate8RightIntoCarry(L), 2),
        0x0E => (AluOp::Rotate16RightIntoCarry(HL), 4),
        0x0F => (AluOp::Rotate8RightIntoCarry(A), 2),

        0x10 => (AluOp::Rotate8LeftThroughCarry(B), 2),
        0x11 => (AluOp::Rotate8LeftThroughCarry(C), 2),
        0x12 => (AluOp::Rotate8LeftThroughCarry(D), 2),
        0x13 => (AluOp::Rotate8LeftThroughCarry(E), 2),
        0x14 => (AluOp::Rotate8LeftThroughCarry(H), 2),
        0x15 => (AluOp::Rotate8LeftThroughCarry(L), 2),
        0x16 => (AluOp::Rotate16LeftThroughCarry(HL), 4),
        0x17 => (AluOp::Rotate8LeftThroughCarry(A), 2),

        0x18 => (AluOp::Rotate8RightThroughCarry(B), 2),
        0x19 => (AluOp::Rotate8RightThroughCarry(C), 2),
        0x1A => (AluOp::Rotate8RightThroughCarry(D), 2),
        0x1B => (AluOp::Rotate8RightThroughCarry(E), 2),
        0x1C => (AluOp::Rotate8RightThroughCarry(H), 2),
        0x1D => (AluOp::Rotate8RightThroughCarry(L), 2),
        0x1E => (AluOp::Rotate16RightThroughCarry(HL), 4),
        0x1F => (AluOp::Rotate8RightThroughCarry(A), 2),

        0x20 => (AluOp::Shift8LeftArithmetic(B), 2),
        0x21 => (AluOp::Shift8LeftArithmetic(C), 2),
        0x22 => (AluOp::Shift8LeftArithmetic(D), 2),
        0x23 => (AluOp::Shift8LeftArithmetic(E), 2),
        0x24 => (AluOp::Shift8LeftArithmetic(H), 2),
        0x25 => (AluOp::Shift8LeftArithmetic(L), 2),
        0x26 => (AluOp::Shift16LeftArithmetic(HL), 4),
        0x27 => (AluOp::Shift8LeftArithmetic(A), 2),

        0x28 => (AluOp::Shift8RightArithmetic(B), 2),
        0x29 => (AluOp::Shift8RightArithmetic(C), 2),
        0x2A => (AluOp::Shift8RightArithmetic(D), 2),
        0x2B => (AluOp::Shift8RightArithmetic(E), 2),
        0x2C => (AluOp::Shift8RightArithmetic(H), 2),
        0x2D => (AluOp::Shift8RightArithmetic(L), 2),
        0x2E => (AluOp::Shift16RightArithmetic(HL), 4),
        0x2F => (AluOp::Shift8RightArithmetic(A), 2),

        0x30 => (AluOp::Swap(B), 2),
        0x31 => (AluOp::Swap(C), 2),
        0x32 => (AluOp::Swap(D), 2),
        0x33 => (AluOp::Swap(E), 2),
        0x34 => (AluOp::Swap(H), 2),
        0x35 => (AluOp::Swap(L), 2),
        0x36 => (AluOp::Swap16(HL), 4),
        0x37 => (AluOp::Swap(A), 2),

        0x38 => (AluOp::Shift8RightLogical(B), 2),
        0x39 => (AluOp::Shift8RightLogical(C), 2),
        0x3A => (AluOp::Shift8RightLogical(D), 2),
        0x3B => (AluOp::Shift8RightLogical(E), 2),
        0x3C => (AluOp::Shift8RightLogical(H), 2),
        0x3D => (AluOp::Shift8RightLogical(L), 2),
        0x3E => (AluOp::Shift16RightLogical(HL), 4),
        0x3F => (AluOp::Shift8RightLogical(A), 2),

        // TODO(slongfield):
        // There is a pattern to the following bit tests/sets/resets, could probably
        // generate these within Rust instead of with an outside helper.
        0x40 => (AluOp::TestBit(B, 0), 2),
        0x41 => (AluOp::TestBit(C, 0), 2),
        0x42 => (AluOp::TestBit(D, 0), 2),
        0x43 => (AluOp::TestBit(E, 0), 2),
        0x44 => (AluOp::TestBit(H, 0), 2),
        0x45 => (AluOp::TestBit(L, 0), 2),
        0x46 => (AluOp::TestBit16(HL, 0), 4),
        0x47 => (AluOp::TestBit(A, 0), 2),

        0x48 => (AluOp::TestBit(B, 1), 2),
        0x49 => (AluOp::TestBit(C, 1), 2),
        0x4A => (AluOp::TestBit(D, 1), 2),
        0x4B => (AluOp::TestBit(E, 1), 2),
        0x4C => (AluOp::TestBit(H, 1), 2),
        0x4D => (AluOp::TestBit(L, 1), 2),
        0x4E => (AluOp::TestBit16(HL, 1), 4),
        0x4F => (AluOp::TestBit(A, 1), 2),

        0x50 => (AluOp::TestBit(B, 2), 2),
        0x51 => (AluOp::TestBit(C, 2), 2),
        0x52 => (AluOp::TestBit(D, 2), 2),
        0x53 => (AluOp::TestBit(E, 2), 2),
        0x54 => (AluOp::TestBit(H, 2), 2),
        0x55 => (AluOp::TestBit(L, 2), 2),
        0x56 => (AluOp::TestBit16(HL, 2), 4),
        0x57 => (AluOp::TestBit(A, 2), 2),

        0x58 => (AluOp::TestBit(B, 3), 2),
        0x59 => (AluOp::TestBit(C, 3), 2),
        0x5A => (AluOp::TestBit(D, 3), 2),
        0x5B => (AluOp::TestBit(E, 3), 2),
        0x5C => (AluOp::TestBit(H, 3), 2),
        0x5D => (AluOp::TestBit(L, 3), 2),
        0x5E => (AluOp::TestBit16(HL, 3), 4),
        0x5F => (AluOp::TestBit(A, 3), 2),

        0x60 => (AluOp::TestBit(B, 4), 2),
        0x61 => (AluOp::TestBit(C, 4), 2),
        0x62 => (AluOp::TestBit(D, 4), 2),
        0x63 => (AluOp::TestBit(E, 4), 2),
        0x64 => (AluOp::TestBit(H, 4), 2),
        0x65 => (AluOp::TestBit(L, 4), 2),
        0x66 => (AluOp::TestBit16(HL, 4), 4),
        0x67 => (AluOp::TestBit(A, 4), 2),

        0x68 => (AluOp::TestBit(B, 5), 2),
        0x69 => (AluOp::TestBit(C, 5), 2),
        0x6A => (AluOp::TestBit(D, 5), 2),
        0x6B => (AluOp::TestBit(E, 5), 2),
        0x6C => (AluOp::TestBit(H, 5), 2),
        0x6D => (AluOp::TestBit(L, 5), 2),
        0x6E => (AluOp::TestBit16(HL, 5), 4),
        0x6F => (AluOp::TestBit(A, 5), 2),

        0x70 => (AluOp::TestBit(B, 6), 2),
        0x71 => (AluOp::TestBit(C, 6), 2),
        0x72 => (AluOp::TestBit(D, 6), 2),
        0x73 => (AluOp::TestBit(E, 6), 2),
        0x74 => (AluOp::TestBit(H, 6), 2),
        0x75 => (AluOp::TestBit(L, 6), 2),
        0x76 => (AluOp::TestBit16(HL, 6), 4),
        0x77 => (AluOp::TestBit(A, 6), 2),

        0x78 => (AluOp::TestBit(B, 7), 2),
        0x79 => (AluOp::TestBit(C, 7), 2),
        0x7A => (AluOp::TestBit(D, 7), 2),
        0x7B => (AluOp::TestBit(E, 7), 2),
        0x7C => (AluOp::TestBit(H, 7), 2),
        0x7D => (AluOp::TestBit(L, 7), 2),
        0x7E => (AluOp::TestBit16(HL, 7), 4),
        0x7F => (AluOp::TestBit(A, 7), 2),

        // Reset Bits
        0x80 => (AluOp::ResetBit(B, 0), 2),
        0x81 => (AluOp::ResetBit(C, 0), 2),
        0x82 => (AluOp::ResetBit(D, 0), 2),
        0x83 => (AluOp::ResetBit(E, 0), 2),
        0x84 => (AluOp::ResetBit(H, 0), 2),
        0x85 => (AluOp::ResetBit(L, 0), 2),
        0x86 => (AluOp::ResetBit16(HL, 0), 4),
        0x87 => (AluOp::ResetBit(A, 0), 2),

        0x88 => (AluOp::ResetBit(B, 1), 2),
        0x89 => (AluOp::ResetBit(C, 1), 2),
        0x8A => (AluOp::ResetBit(D, 1), 2),
        0x8B => (AluOp::ResetBit(E, 1), 2),
        0x8C => (AluOp::ResetBit(H, 1), 2),
        0x8D => (AluOp::ResetBit(L, 1), 2),
        0x8E => (AluOp::ResetBit16(HL, 1), 4),
        0x8F => (AluOp::ResetBit(A, 1), 2),

        0x90 => (AluOp::ResetBit(B, 2), 2),
        0x91 => (AluOp::ResetBit(C, 2), 2),
        0x92 => (AluOp::ResetBit(D, 2), 2),
        0x93 => (AluOp::ResetBit(E, 2), 2),
        0x94 => (AluOp::ResetBit(H, 2), 2),
        0x95 => (AluOp::ResetBit(L, 2), 2),
        0x96 => (AluOp::ResetBit16(HL, 2), 4),
        0x97 => (AluOp::ResetBit(A, 2), 2),

        0x98 => (AluOp::ResetBit(B, 3), 2),
        0x99 => (AluOp::ResetBit(C, 3), 2),
        0x9A => (AluOp::ResetBit(D, 3), 2),
        0x9B => (AluOp::ResetBit(E, 3), 2),
        0x9C => (AluOp::ResetBit(H, 3), 2),
        0x9D => (AluOp::ResetBit(L, 3), 2),
        0x9E => (AluOp::ResetBit16(HL, 3), 4),
        0x9F => (AluOp::ResetBit(A, 3), 2),

        0xA0 => (AluOp::ResetBit(B, 4), 2),
        0xA1 => (AluOp::ResetBit(C, 4), 2),
        0xA2 => (AluOp::ResetBit(D, 4), 2),
        0xA3 => (AluOp::ResetBit(E, 4), 2),
        0xA4 => (AluOp::ResetBit(H, 4), 2),
        0xA5 => (AluOp::ResetBit(L, 4), 2),
        0xA6 => (AluOp::ResetBit16(HL, 4), 4),
        0xA7 => (AluOp::ResetBit(A, 4), 2),

        0xA8 => (AluOp::ResetBit(B, 5), 2),
        0xA9 => (AluOp::ResetBit(C, 5), 2),
        0xAA => (AluOp::ResetBit(D, 5), 2),
        0xAB => (AluOp::ResetBit(E, 5), 2),
        0xAC => (AluOp::ResetBit(H, 5), 2),
        0xAD => (AluOp::ResetBit(L, 5), 2),
        0xAE => (AluOp::ResetBit16(HL, 5), 4),
        0xAF => (AluOp::ResetBit(A, 5), 2),

        0xB0 => (AluOp::ResetBit(B, 6), 2),
        0xB1 => (AluOp::ResetBit(C, 6), 2),
        0xB2 => (AluOp::ResetBit(D, 6), 2),
        0xB3 => (AluOp::ResetBit(E, 6), 2),
        0xB4 => (AluOp::ResetBit(H, 6), 2),
        0xB5 => (AluOp::ResetBit(L, 6), 2),
        0xB6 => (AluOp::ResetBit16(HL, 6), 4),
        0xB7 => (AluOp::ResetBit(A, 6), 2),

        0xB8 => (AluOp::ResetBit(B, 7), 2),
        0xB9 => (AluOp::ResetBit(C, 7), 2),
        0xBA => (AluOp::ResetBit(D, 7), 2),
        0xBB => (AluOp::ResetBit(E, 7), 2),
        0xBC => (AluOp::ResetBit(H, 7), 2),
        0xBD => (AluOp::ResetBit(L, 7), 2),
        0xBE => (AluOp::ResetBit16(HL, 7), 4),
        0xBF => (AluOp::ResetBit(A, 7), 2),

        // Set bits.
        0xC0 => (AluOp::SetBit(B, 0), 2),
        0xC1 => (AluOp::SetBit(C, 0), 2),
        0xC2 => (AluOp::SetBit(D, 0), 2),
        0xC3 => (AluOp::SetBit(E, 0), 2),
        0xC4 => (AluOp::SetBit(H, 0), 2),
        0xC5 => (AluOp::SetBit(L, 0), 2),
        0xC6 => (AluOp::SetBit16(HL, 0), 4),
        0xC7 => (AluOp::SetBit(A, 0), 2),

        0xC8 => (AluOp::SetBit(B, 1), 2),
        0xC9 => (AluOp::SetBit(C, 1), 2),
        0xCA => (AluOp::SetBit(D, 1), 2),
        0xCB => (AluOp::SetBit(E, 1), 2),
        0xCC => (AluOp::SetBit(H, 1), 2),
        0xCD => (AluOp::SetBit(L, 1), 2),
        0xCE => (AluOp::SetBit16(HL, 1), 4),
        0xCF => (AluOp::SetBit(A, 1), 2),

        0xD0 => (AluOp::SetBit(B, 2), 2),
        0xD1 => (AluOp::SetBit(C, 2), 2),
        0xD2 => (AluOp::SetBit(D, 2), 2),
        0xD3 => (AluOp::SetBit(E, 2), 2),
        0xD4 => (AluOp::SetBit(H, 2), 2),
        0xD5 => (AluOp::SetBit(L, 2), 2),
        0xD6 => (AluOp::SetBit16(HL, 2), 4),
        0xD7 => (AluOp::SetBit(A, 2), 2),

        0xD8 => (AluOp::SetBit(B, 3), 2),
        0xD9 => (AluOp::SetBit(C, 3), 2),
        0xDA => (AluOp::SetBit(D, 3), 2),
        0xDB => (AluOp::SetBit(E, 3), 2),
        0xDC => (AluOp::SetBit(H, 3), 2),
        0xDD => (AluOp::SetBit(L, 3), 2),
        0xDE => (AluOp::SetBit16(HL, 3), 4),
        0xDF => (AluOp::SetBit(A, 3), 2),

        0xE0 => (AluOp::SetBit(B, 4), 2),
        0xE1 => (AluOp::SetBit(C, 4), 2),
        0xE2 => (AluOp::SetBit(D, 4), 2),
        0xE3 => (AluOp::SetBit(E, 4), 2),
        0xE4 => (AluOp::SetBit(H, 4), 2),
        0xE5 => (AluOp::SetBit(L, 4), 2),
        0xE6 => (AluOp::SetBit16(HL, 4), 4),
        0xE7 => (AluOp::SetBit(A, 4), 2),

        0xE8 => (AluOp::SetBit(B, 5), 2),
        0xE9 => (AluOp::SetBit(C, 5), 2),
        0xEA => (AluOp::SetBit(D, 5), 2),
        0xEB => (AluOp::SetBit(E, 5), 2),
        0xEC => (AluOp::SetBit(H, 5), 2),
        0xED => (AluOp::SetBit(L, 5), 2),
        0xEE => (AluOp::SetBit16(HL, 5), 4),
        0xEF => (AluOp::SetBit(A, 5), 2),

        0xF0 => (AluOp::SetBit(B, 6), 2),
        0xF1 => (AluOp::SetBit(C, 6), 2),
        0xF2 => (AluOp::SetBit(D, 6), 2),
        0xF3 => (AluOp::SetBit(E, 6), 2),
        0xF4 => (AluOp::SetBit(H, 6), 2),
        0xF5 => (AluOp::SetBit(L, 6), 2),
        0xF6 => (AluOp::SetBit16(HL, 6), 4),
        0xF7 => (AluOp::SetBit(A, 6), 2),

        0xF8 => (AluOp::SetBit(B, 7), 2),
        0xF9 => (AluOp::SetBit(C, 7), 2),
        0xFA => (AluOp::SetBit(D, 7), 2),
        0xFB => (AluOp::SetBit(E, 7), 2),
        0xFC => (AluOp::SetBit(H, 7), 2),
        0xFD => (AluOp::SetBit(L, 7), 2),
        0xFE => (AluOp::SetBit16(HL, 7), 4),
        0xFF => (AluOp::SetBit(A, 7), 2),

        // Needed to satisfy exhaustive checker, but completely unreachable.
        _ => panic!("Invalid opcode!"),
    };

    (Op::AluOp(alu_op), 2, time)
}
