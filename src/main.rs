extern crate env_logger;
extern crate structopt;

extern crate wolfwig;

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

    /// Should the emulator start in debug mode
    #[structopt(short = "d", long = "debug")]
    debug: bool,

    /// Should bytes printed sent out the serial port be printed to the console?
    #[structopt(short = "p", long = "print_serial")]
    print_serial: bool,

    /// Should the emulator go fast (i.e., ignore all speed limits?).
    #[structopt(short = "f", long = "go_fast")]
    go_fast: bool,
}

fn main() {
    env_logger::init();
    let opt = Opt::from_args();
    let mut wolfwig = wolfwig::Wolfwig::from_files(&opt.bootrom, &opt.rom).unwrap();
    if opt.print_serial {
        wolfwig.start_print_serial()
    }
    if opt.go_fast {
        wolfwig.go_fast();
    }

    wolfwig.print_header();

    if opt.debug {
        let mut debug = wolfwig::debug::Debug::new(wolfwig);
        loop {
            debug.step();
        }
    } else {
        loop {
            wolfwig.step();
        }
    }
}
