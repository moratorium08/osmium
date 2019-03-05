#!/bin/sh

cat kernel/bin/osmium.bin.lengthed - | tools/bin/emu bootloader/bin/bootloader.bin $@
