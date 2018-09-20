// 8-bit registers.
#[derive(Debug)]
pub enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

// 16-bit registers.
#[derive(Debug)]
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

pub enum Flags {
    // True if the previous result was zero
    Zero,
    // True if the previous operation was subtract.
    Subtract,
    // True if the carry-out was true for the lower 4 bits of the previous op.
    HalfCarry,
    // True if the carry-out was true for the lower 8 bits of the previous op.
    Carry,
}

///! Structure that holds the current register values from the CPU.
pub struct Registers {
    a: u8,
    f: u8, // TODO(slongfield): Special p:urpose flag type.
    b: u8,
    c: u8,
    d: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
}
