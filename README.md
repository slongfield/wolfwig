# wolfwig

Wolfwig is a GameBoy emulator written in Rust.

The current primary goal for Wolfwig is for me to learn about Gameboy emulation, and learn Rust.

## Features

No features have been implemented.

## Tests

### Blargg Test Roms

#### `cpu_insts`

- [x] 01 - Special
- [ ] 02 - Interrupts
- [x] 03 - Op SP, HL
- [x] 04 - Op R, IMM
- [x] 05 - Op RP
- [x] 06 - LD R, r.s
- [x] 07 - Jr,Jp,Call,Ret,rst.s
- [x] 08 - Misc
- [ ] 09 - Op r,r.s
- [x] 10 - bit ops
- [x] 11 - op a,(hl).s

## TODO

- [ ] Emulate basic LR25902
- [ ] Timer peripheral
- [ ] Basic PPU (Tetris-level)
- [ ] Joypad
- [ ] Handle Memory Controllers
- [ ] APU
