extern crate wolfwig;

use std::env;
use std::path::Path;
use std::process;

fn main() {
    let mut args = env::args();
    args.next().unwrap();
    let filename = match args.next() {
        Some(file) => file,
        None => {
            eprintln!("Must pass a rom to load!");
            process::exit(1);
        }
    };
    let wolfwig = wolfwig::Wolfwig::from_file(&Path::new(&filename)).unwrap();
    wolfwig.print_header();
    wolfwig.dump_instructions(0x150, 0x200);
}
