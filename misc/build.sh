#!/bin/sh

# できればcargoで完結させたいが、一旦めんどくさいので
env CC=riscv32-unknown-linux-gnu-gcc cargo xbuild --target riscv32ima-unknown-none-elf.json -vv
