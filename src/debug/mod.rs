/// The Debug module has a very simple, gdb-inspired debugger that the emulator can run under. This
/// is mostly designed for debugging the emulator itself while it's under development.
use Wolfwig;

use cpu::decode;
use cpu::registers;
use std::collections::HashSet;
use std::io::{stdin, stdout, Write};
use std::iter::Iterator;
use std::process;

pub struct Debug {
    wolfwig: Wolfwig,
    cycle: usize,
    pc: u16,
    last_pc: u16,
    run: usize,
    steps: u32,
    breakpoints: HashSet<u16>,
    verbose: bool,
    frame: u32,
    wait_for_frame: bool,
}

const HELP: &str = "Available commands:
 [n]ext n     -- Runs the next n instructions, default 1 if nothing is provided
 [f]rame      -- Runs until the start of the next frame
 [b]reakpoint -- Sets a breakpoint
 [i]nfo       -- lists breakpoins
 [d]elete     -- deletes a breakpoint
 [r]un n      -- Run freely, until breakpoint, n times. Default 1.
 [p]rint      -- register name prints specific register, 0xNNNN prints memory address,
                 blank prints all registers.
 [v]erbose   -- enable verbose printing of instruction stream
 [q]uit       -- quit";

fn to_int32(s: &str) -> Option<u32> {
    if let Ok(d) = s.parse::<u32>() {
        return Some(d);
    }
    if &s[0..2] == "0x" {
        if let Ok(d) = u32::from_str_radix(&s[2..], 16) {
            return Some(d);
        }
    }
    None
}

fn next_as_int32(iter: &mut Iterator<Item = &str>) -> Option<u32> {
    if let Some(val) = iter.next() {
        if let Some(parsed) = to_int32(val) {
            return Some(parsed);
        }
        println!("Could not parse {}", val);
    }
    None
}

impl Debug {
    pub fn new(wolfwig: Wolfwig) -> Self {
        Self {
            wolfwig,
            cycle: 0,
            pc: 0,
            last_pc: 0,
            run: 0,
            steps: 0,
            breakpoints: HashSet::new(),
            verbose: false,
            frame: 0,
            wait_for_frame: false,
        }
    }

    pub fn step(&mut self) -> u16 {
        self.wolfwig.step();
        self.pc = self.wolfwig.pc();
        if self.pc != self.last_pc && self.run != 0 {
            if self.breakpoints.contains(&self.pc) {
                self.run -= 1;
            } else if self.verbose {
                let (op, _, _) = decode::decode(&self.wolfwig.peripherals, self.pc);
                println!(
                    "PC: 0x{:02X} Cycle: 0x{:04X} Op: {}",
                    self.pc, self.cycle, op
                );
            }
        }
        if self.pc != self.last_pc && self.run == 0 {
            let (op, _, _) = decode::decode(&self.wolfwig.peripherals, self.pc);
            println!(
                "PC: 0x{:02X} Cycle: 0x{:04X} Op: {}",
                self.pc, self.cycle, op
            );
            if (self.wait_for_frame && self.frame > self.wolfwig.peripherals.ppu.frame) {
            } else if (self.wait_for_frame) {
                self.wait_for_frame = false;
            } else if self.steps > 0 {
                self.steps -= 1;
            } else {
                self.prompt()
            }
        }
        self.last_pc = self.pc;
        self.cycle += 1;
        self.pc
    }

    fn prompt(&mut self) {
        loop {
            let mut buf = String::new();
            print!("> ");
            stdout().flush().expect("Could not flush stdout");
            stdin().read_line(&mut buf).unwrap();
            let mut split = buf.trim_end().split(' ');
            match split.next() {
                Some("r") | Some("run") => {
                    if let Some(times) = next_as_int32(&mut split) {
                        self.run = times as usize;
                    } else {
                        self.run = 1;
                    }
                    break;
                }
                Some("n") | Some("next") | Some("") => {
                    if let Some(steps) = next_as_int32(&mut split) {
                        self.steps = steps;
                    };
                    break;
                }
                Some("f") | Some("frame") => {
                    self.frame = self.wolfwig.peripherals.ppu.frame + 1;
                    self.wait_for_frame = true;
                    break;
                }
                Some("b") | Some("breakpoint") => {
                    if let Some(pc) = next_as_int32(&mut split) {
                        self.breakpoints.insert(pc as u16);
                    }
                }
                Some("d") | Some("delete") => {
                    if let Some(pc) = next_as_int32(&mut split) {
                        self.breakpoints.remove(&(pc as u16));
                    }
                }
                Some("i") | Some("info") => println!("{:?}", self.breakpoints),
                Some("h") | Some("help") => println!("{}", HELP),
                Some("p") | Some("print") => match split.next() {
                    Some("A") => self.wolfwig.print_reg8(registers::Reg8::A),
                    Some("B") => self.wolfwig.print_reg8(registers::Reg8::B),
                    Some("C") => self.wolfwig.print_reg8(registers::Reg8::C),
                    Some("D") => self.wolfwig.print_reg8(registers::Reg8::D),
                    Some("E") => self.wolfwig.print_reg8(registers::Reg8::E),
                    Some("H") => self.wolfwig.print_reg8(registers::Reg8::H),
                    Some("L") => self.wolfwig.print_reg8(registers::Reg8::L),
                    Some("AF") => self.wolfwig.print_reg16(registers::Reg16::AF),
                    Some("BC") => self.wolfwig.print_reg16(registers::Reg16::BC),
                    Some("DE") => self.wolfwig.print_reg16(registers::Reg16::DE),
                    Some("HL") => self.wolfwig.print_reg16(registers::Reg16::HL),
                    Some("SP") => self.wolfwig.print_reg16(registers::Reg16::SP),
                    Some("PC") => self.wolfwig.print_reg16(registers::Reg16::PC),
                    Some(val) => match to_int32(val) {
                        Some(addr) if addr <= 0xFFFF => {
                            println!("0x{:02X}", self.wolfwig.peripherals.read(addr as u16))
                        }
                        Some(addr) => println!("Addr 0x{:X} too large", addr),
                        None => {
                            let mut range = val.split('-');
                            if let (Some(start), Some(end)) =
                                (next_as_int32(&mut range), next_as_int32(&mut range))
                            {
                                print!("[");
                                for addr in start..(end + 1) {
                                    print!(" 0x{:02X}", self.wolfwig.peripherals.read(addr as u16));
                                }
                                println!(" ]");
                            } else {
                                println!("Could not parse {}", val);
                            }
                        }
                    },
                    None => {
                        self.wolfwig.print_registers();
                        println!(
                            "{} {} {}",
                            self.wait_for_frame, self.frame, self.wolfwig.peripherals.ppu.frame
                        )
                    }
                },
                Some("v") | Some("verbose") => self.verbose = !self.verbose,
                Some("q") | Some("quit") => process::exit(0),
                cmd => println!(
                    "Unrecognized command: {:?}. Type 'help' for valid comamnds",
                    cmd
                ),
            }
        }
    }
}
