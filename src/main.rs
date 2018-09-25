#[macro_use]
extern crate structopt;

extern crate wolfwig;

use std::io::stdin;
use std::path::PathBuf;
use structopt::StructOpt;

/// The Wolfwig gameboy emulator.
#[derive(StructOpt)]
struct Opt {
    /// ROM to load
    #[structopt(short = "r", long = "rom", parse(from_os_str))]
    rom: PathBuf,

    /// Bootrom
    #[structopt(short = "b", long = "bootrom", parse(from_os_str))]
    bootrom: PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    let mut wolfwig = wolfwig::Wolfwig::from_files(&opt.bootrom, &opt.rom).unwrap();
    wolfwig.print_header();

    let mut pc = 0;
    let mut buf = String::new();
    loop {
        pc = wolfwig.step();
        println!("Executed pc {}", pc);
        stdin().read_line(&mut buf).unwrap();
        if pc == 0x100 {
            break;
        }
    }
}
