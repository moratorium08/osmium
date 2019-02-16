#!/bin/sh

mkdir -p bin

env CC=riscv32-unknown-linux-gnu-gcc cargo xbuild --target riscv32ima-unknown-none-elf.json -vv  --out-dir bin
