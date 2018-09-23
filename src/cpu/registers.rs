use std::fmt;

// 8-bit registers.
#[derive(Debug, Copy, Clone)]
pub enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl fmt::Display for Reg8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// 16-bit registers.
#[derive(Debug, Copy, Clone)]
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

impl fmt::Display for Reg16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Flag bits
const ZERO_BIT: u8 = 1 << 7;
const SUBTRACT_BIT: u8 = 1 << 6;
const HALF_CARRY_BIT: u8 = 1 << 5;
const CARRY_BIT: u8 = 1 << 4;

#[derive(Debug, Copy, Clone)]
pub enum Flag {
    // True if the previous result was zero
    Zero,
    NotZero,
    // True if the previous operation was subtract.
    Subtract,
    // True if the carry-out was true for the lower 4 bits of the previous op.
    HalfCarry,
    // True if the carry-out was true for the lower 8 bits of the previous op.
    Carry,
    NotCarry,
}

impl fmt::Display for Flag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

///! Structure that holds the current register values from the CPU.
pub struct Registers {
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }

    pub fn read8(&self, r: Reg8) -> u8 {
        match r {
            Reg8::A => self.a,
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::E => self.e,
            Reg8::H => self.h,
            Reg8::L => self.l,
        }
    }

    pub fn read16(&self, r: Reg16) -> u16 {
        match r {
            Reg16::AF => u16::from(self.a) << 8 & u16::from(self.f),
            Reg16::BC => u16::from(self.b) << 8 & u16::from(self.c),
            Reg16::DE => u16::from(self.d) << 8 & u16::from(self.e),
            Reg16::HL => u16::from(self.h) << 8 & u16::from(self.l),
            Reg16::SP => self.sp,
            Reg16::PC => self.pc,
        }
    }

    pub fn read_flag(&self, f: Flag) -> bool {
        match f {
            Flag::Zero => (self.f & ZERO_BIT) != 0,
            Flag::NotZero => (self.f & ZERO_BIT) == 0,
            Flag::Subtract => (self.f & SUBTRACT_BIT) != 0,
            Flag::HalfCarry => (self.f & HALF_CARRY_BIT) != 0,
            Flag::Carry => (self.f & CARRY_BIT) != 0,
            Flag::NotCarry => (self.f & CARRY_BIT) == 0,
        }
    }

    pub fn set8(&mut self, r: Reg8, data: u8) {
        match r {
            Reg8::A => self.a = data,
            Reg8::B => self.b = data,
            Reg8::C => self.c = data,
            Reg8::D => self.d = data,
            Reg8::E => self.e = data,
            Reg8::H => self.h = data,
            Reg8::L => self.l = data,
        }
    }

    pub fn set16(&mut self, r: Reg16, data: u16) {
        match r {
            Reg16::AF => {
                self.a = (data >> 8) as u8;
                self.f = data as u8;
            }
            Reg16::BC => {
                self.b = (data >> 8) as u8;
                self.c = data as u8;
            }
            Reg16::DE => {
                self.d = (data >> 8) as u8;
                self.e = data as u8;
            }
            Reg16::HL => {
                self.h = (data >> 8) as u8;
                self.l = data as u8;
            }
            Reg16::SP => self.sp = data,
            Reg16::PC => self.pc = data,
        }
    }

    pub fn set_flag(&mut self, f: Flag, val: bool) {
        match (f, val) {
            (Flag::Zero, true) => self.f |= ZERO_BIT,
            (Flag::Zero, false) => self.f &= !ZERO_BIT,
            (Flag::Subtract, true) => self.f |= SUBTRACT_BIT,
            (Flag::Subtract, false) => self.f &= !SUBTRACT_BIT,
            (Flag::HalfCarry, true) => self.f |= HALF_CARRY_BIT,
            (Flag::HalfCarry, false) => self.f &= !HALF_CARRY_BIT,
            (Flag::Carry, true) => self.f |= CARRY_BIT,
            (Flag::Carry, false) => self.f &= !CARRY_BIT,
            // TODO(slongfield): Could fix this, but shouldn't need it.
            _ => panic!("Cannot set the negated forms of flags."),
        }
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "A: 0x{:X} B: 0x{:X} C: 0x{:X} D: 0x{:X} E: 0x{:X} H: 0x{:X} L: 0x{:X}",
            self.read8(Reg8::A),
            self.read8(Reg8::B),
            self.read8(Reg8::C),
            self.read8(Reg8::D),
            self.read8(Reg8::E),
            self.read8(Reg8::H),
            self.read8(Reg8::L)
        )?;
        writeln!(
            f,
            "AF: 0x{:X} BC: 0x{:X} DE: 0x{:X} HL: 0x{:X} SP: 0x{:X} PC: 0x{:X}",
            self.read16(Reg16::AF),
            self.read16(Reg16::BC),
            self.read16(Reg16::DE),
            self.read16(Reg16::HL),
            self.read16(Reg16::SP),
            self.read16(Reg16::PC)
        )?;
        writeln!(
            f,
            "Zero: {} Sub: {} HalfCarry: {} Carry: {}",
            self.read_flag(Flag::Zero),
            self.read_flag(Flag::Subtract),
            self.read_flag(Flag::HalfCarry),
            self.read_flag(Flag::Carry)
        )
    }
}
