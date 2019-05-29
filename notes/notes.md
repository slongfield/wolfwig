# slongfield@'s notes on Rust programming and Gameboy

## Rust commands to remember:

Lint:

 * `cargo clean; cargo clippy -- -W clippy::pedantic`

Build:

 * `cargo build --release`

Run:

 * `./target/release/wolfwig -b bootroms/DMG_ROM.bin --rom roms/Tetris\ \(Japan\)\ \(En\).gb`

Logging:

 * Using env\_logger. Need to have the full path when enabling the logs, e.g.;
```
RUST_LOG=wolfwig::peripherals::timer=debug ./target/release/wolfwig
```
   Will enable debug logging in the timer. Won't work without the leading `wolfwig::`.


## Useful resources:

Gekkio's Mooneye GB:
  * https://github.com/Gekkio/mooneye-gb

## Gameboy reminders

 * If a game senses that select+start are pressed at the same time, it will attempt a 'soft reset',
   which may look very odd.
