#!/bin/sh

mkdir -p bin
mv _build.rs build.rs
env CC=riscv32-unknown-linux-gnu-gcc cargo xbuild --target riscv32ima-unknown-none-elf.json -Z unstable-options  --out-dir bin
mv build.rs _build.rs
