# wolfwig

Wolfwig is a GameBoy emulator written in Rust.

The current primary goal for Wolfwig is for me to learn about Gameboy emulation, and learn Rust.

## Features

No features have been implemented.

## Tests

### Blargg Test Roms

#### `cpu_insts`

- [x] 01 - Special
- [x] 02 - Interrupts
- [x] 03 - Op SP, HL
- [x] 04 - Op R, IMM
- [x] 05 - Op RP
- [x] 06 - LD R, r.s
- [x] 07 - Jr,Jp,Call,Ret,rst.s
- [x] 08 - Misc
- [x] 09 - Op r,r.s
- [x] 10 - bit ops
- [x] 11 - op a,(hl).s

Note: All the individual tests pass, but the full test does not. Possibly due to lack of modeling
for memory controllers?

## TODO

- [x] Emulate basic LR25902
- [x] Timer peripheral
- [ ] Basic PPU (Tetris-level)
- [ ] Joypad
- [ ] Handle Memory Controllers
- [ ] APU
