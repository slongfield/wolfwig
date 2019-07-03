# Configuration

This directory contains some configuration files for generating code and documentation for the Wolfwig emulator.

## State

### Format

Currently very ad-hoc YAML, basically my notes on the state.
 * Would System-RDL be _way_ overkill, or just ~mostly~ overkill?

### Operations that the generated code needs to support:

 * Creating the core state
 * Creating software read and write ports
 * Creating interfaces that can be passed to the different emulated hardware modules for them to
   access the state
 * Sutting up callbacks to happen on writes and reads, if they're not just plain values
   * E.g., Certain addresses hold bank offset information
 * Run-time configurability, as some things (e.g., # of banks of memory) depend on the cartridge
   type, and others depend on what mode the system is running in (e.g., DMG vs CGB)

### Gameboy State file

`gameboy.yaml`

This file contains notes on the addressable and non-addressible state contained in the Game Boy.
Originally written to match the DMG1.
