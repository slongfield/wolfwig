# DevLog

These are raw notes and logs of what I was doing and what I was thinking at the time.

## Ideas

### 2010-05-27

* Using mooneye/misc/bits/unused\_hwio to check that the new registers are hooked up correctly.
* Time gets away from me quickly.
* Screwed something up, and Tetris started flashing as soon as it loaded. But what?
  * Apparently, not calling joypad.update() on write screwed it up. I guess it was looking for
    those values to get cleared?

### 2019-05-08

* Starting to get back into this after the move. Next goal is to improve the register access by
  using a macro of some kind. Want to move the addressing out of individual modules.
* Wrote a simple prototype in serial. Seems like it'll generalize eaisly. It does mean that the
  public API of each of the peripherals is a bit larger, and now the register map will happen in
  src/peripherals/mod.rs, but I think that makes more sense than happening in each file
  individually.
