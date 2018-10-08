# Testroms

This directory contains my test roms, they're designed to be compiled with wla-dx.

## TODOs

 *  Evalute RGBDS
 *  Write up a simple howto for wla-dx. `sub.s` was cargo-culted from things I found online.

## How to build

`wla-gb -o test.o sub.s && wlalink linkfile sub.gb`

TODO(slongfield): Put together a Makefile, or something like that.

