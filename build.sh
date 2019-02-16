#!/bin/sh

# できればcargoで完結させたいが、一旦めんどくさいので
cd misc
./build.sh
cd ..

cd bootloader
./build.sh
cd ..

mkdir -p bin
mv _build.rs build.rs
env CC=riscv32-unknown-linux-gnu-gcc cargo xbuild --target riscv32ima-unknown-none-elf.json -vv -Z unstable-options  --out-dir bin
mv build.rs _build.rs
