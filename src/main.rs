#[macro_use]
extern crate structopt;
extern crate env_logger;

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
}

fn main() {
    env_logger::init();
    let opt = Opt::from_args();
    let mut wolfwig = wolfwig::Wolfwig::from_files(&opt.bootrom, &opt.rom).unwrap();
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
