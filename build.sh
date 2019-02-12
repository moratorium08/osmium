#!/bin/sh

# できればcargoで完結させたいが、一旦めんどくさいので
cd misc
./build.sh
cd ..
mv _build.rs build.rs
env CC=riscv32-unknown-linux-gnu-gcc cargo xbuild --target riscv32ima-unknown-none-elf.json -vv
mv build.rs _build.rs
