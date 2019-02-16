#!/bin/sh

cat bin/osmium.bin.lengthed - | tools/bin/emu bootloader/bin/bootloader.bin $@
