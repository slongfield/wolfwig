# DevLog

These are raw notes and logs of what I was doing and what I was thinking at the time.

## TODO

This is the section for active TODOs.

* Basic APU
* Double-tall sprites
* APU ABC output
* Generated state management


## Ideas

### Scripting

* I should add a scripting interface
* romhacking.net
 * Looks like colorizing games is a popular hack.

### Modules

#### Pixel Processing Unit

* Could we implement something like [WideNes](http://prilik.com/blog/2018/08/24/wideNES.html) for
  the gameboy?

#### Audio Processing Unit

* Can the APU output notation?
  * ABC notation language looks like it'd be the simplest.

### State

* Currently, the state is distributed throughout the peripherals. Would it make more sense if we put
  all of the state configuration into a YAML file, and then generated a state object that could be
  used everywhere?
  * `build.rs` gives control over the build process, could do this there.
  * Could also generate some documentation for me that I like more than the existing documentation.

## Log

### 2019-07-20

 * Working on audio.
 * When sending sterep audio to SDL2, it expects the channels to be interleaved, starting with
   the left channel.

### 2019-07-04

 * PPU window
 * Problem with Super Mario World was overactive timer--apparently it sets a timer when it's
   loading the first level, and waits for the interrupt. But if the interrupt keeps re-triggering,
   it'll dump you back to the main screen. Seems like maybe the next interrupt is the level timer,
   so you keep losing lives?
   * Without sprite priority, you can see all the 'hidden' enemies :D
   * Level 1-3 Piranha plants are 'in front' of the pipes.

### 2019-07-03

 * Working on PPU.

### 2019-07-01

 * Finally done with that refactor. Now I want to do another one.

### 2019-06-29

 * More registers.
   * I already dislike this scheme. Better than what we were doing before, but the encapsulation
     will make some of the flexability I want to add harder to implement.
   * Want to get it working before I try something different, though.
 * Got past level 10 on Tetris and saw the rocket. The sprites were messed up, so only half of the
   rocket was properly loaded.

### 2019-06-19

* There are so many registers.

### 2019-05-29

* I wonder if it's worth it to make a generator for these registers?

### 2019-05-28

* Applying that new macro bit by bit.
* Would be nice to have a macro to create getters and setters, but apparently can't concatenate
  names together in macros. :/

### 2019-05-27

* Using mooneye/misc/bits/unused\_hwio to check that the new registers are hooked up correctly.
* Time gets away from me quickly.
* Screwed something up, and Tetris started flashing as soon as it loaded. But what?
  * Apparently, not calling joypad.update() on write screwed it up. I guess it was looking for
    those values to get cleared?
  * Bad joypad update caused it to appear like start and select were held down, which soft-resets
    the system, and Tetris respects that soft-reset.

### 2019-05-08

* Starting to get back into this after the move. Next goal is to improve the register access by
  using a macro of some kind. Want to move the addressing out of individual modules.
* Wrote a simple prototype in serial. Seems like it'll generalize easily. It does mean that the
  public API of each of the peripherals is a bit larger, and now the register map will happen in
  src/peripherals/mod.rs, but I think that makes more sense than happening in each file
  individually.
