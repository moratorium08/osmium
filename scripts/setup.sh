#!/bin/sh

rm -rf tools
git clone --depth 1 --recursive https://github.com/cpu-3/tools
make emu_build -C tools
make elf2bin_build -C tools
