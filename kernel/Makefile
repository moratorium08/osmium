BIN := "../tools/bin"

binary:
	mkdir -p bin
	mv _build.rs build.rs
	env CC=riscv32-unknown-linux-gnu-gcc cargo xbuild --target riscv32ima-unknown-none-elf.json -Z unstable-options --out-dir bin
	mv build.rs _build.rs

elf2bin: binary
	$(BIN)/elf2bin kernel/bin/osmium bin/osmium.bin
	$(BIN)/elf2bin bootloader/bin/bootloader bootloader/bin/bootloader.bin

	python append_length.py

build: binary
