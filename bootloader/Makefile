BIN := "../tools/bin"

binary:
	mkdir -p bin
	env CC=riscv32-unknown-linux-gnu-gcc cargo xbuild --target riscv32ima-unknown-none-elf.json -Z unstable-options  --out-dir bin

.PHONY: binary

elf2bin: binary
	$(BIN)/elf2bin bin/bootloader bin/bootloader.bin
.PHONY: elf2bin

build: elf2bin 
