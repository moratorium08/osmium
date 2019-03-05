tools/bin/elf2bin kernel/bin/osmium bin/osmium.bin
tools/bin/elf2bin bootloader/bin/bootloader bootloader/bin/bootloader.bin

python append_length.py
