/// The Debug module has a very simple, gdb-inspired debugger that the emulator can run under. This
/// is mostly designed for debugging the emulator itself while it's under development.
use Wolfwig;

use cpu::decode;
use cpu::registers;
use std::collections::HashSet;
use std::io::{stdin, stdout, Write};
use std::process;

pub struct Debug {
    wolfwig: Wolfwig,
    cycle: usize,
    pc: u16,
    last_pc: u16,
    run: bool,
    steps: u32,
    breakpoints: HashSet<u16>,
}

const HELP: &str = "Available commands:
 [n]ext n     -- Runs the next n instructions, default 1 if nothing is provided
 [b]reakpoint -- Sets a breakpoint
 [i]nfo       -- lists breakpoins
 [d]elete     -- deletes a breakpoint
 [r]un        -- Run freely, until breakpoint.
 [p]rint      -- register name prints specific register, 0xNNNN prints memory address,
                 blank prints all registers.
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

impl Debug {
    pub fn new(wolfwig: Wolfwig) -> Debug {
        Debug {
            wolfwig,
            cycle: 0,
            pc: 0,
            last_pc: 0,
            run: false,
            steps: 0,
            breakpoints: HashSet::new(),
        }
    }

    pub fn step(&mut self) -> u16 {
        self.pc = self.wolfwig.step();
        if self.pc != self.last_pc && self.run {
            if self.breakpoints.contains(&self.pc) {
                println!("Hit breakpoint {}", self.pc);
                self.run = false;
            }
        }
        if self.pc != self.last_pc && !self.run {
            let (op, _, _) = decode::decode(&self.wolfwig.mem, self.pc as usize);
            println!(
                "PC: 0x{:02X} Cycle: 0x{:04X} Op: {}",
                self.pc, self.cycle, op
            );
            if self.steps > 0 {
                self.steps -= 1;
            } else {
                let mut buf = String::new();
                print!("> ");
                stdout().flush().ok().expect("Could not flush stdout");
                stdin().read_line(&mut buf).unwrap();
                let mut split = buf.trim_right().split(" ");
                match split.next() {
                    Some("r") | Some("run") => self.run = true,
                    Some("n") | Some("next") | Some("") => match split.next() {
                        Some(val) => match to_int32(val) {
                            Some(steps) => self.steps = steps,
                            _ => println!("Could not parse {}", val),
                        },
                        _ => {}
                    },
                    Some("b") | Some("breakpoint") => match split.next() {
                        Some(val) => match to_int32(val) {
                            Some(pc) => {
                                self.breakpoints.insert(pc as u16);
                            }
                            _ => println!("Could not parse {}", val),
                        },
                        _ => {}
                    },
                    Some("d") | Some("delete") => match split.next() {
                        Some(val) => match to_int32(val) {
                            Some(pc) => {
                                self.breakpoints.remove(&(pc as u16));
                            }
                            _ => println!("Could not parse {}", val),
                        },
                        _ => {}
                    },
                    Some("i") | Some("info") => println!("{:?}", self.breakpoints),
                    Some("h") | Some("help") => println!("{}", HELP),
                    Some("p") | Some("print") => match split.next() {
                        Some("A") => self.wolfwig.print_reg8(registers::Reg8::A),
                        None => self.wolfwig.print_registers(),
                        Some(other) => println!("Uknown register or memory address: {}", other),
                    },
                    Some("q") | Some("quit") => process::exit(0),
                    cmd => println!(
                        "Unrecognized command: {:?}. Type 'help' for valid comamnds",
                        cmd
                    ),
                }
            }
        }
        self.last_pc = self.pc;
        self.cycle += 1;
        self.pc
    }
}
