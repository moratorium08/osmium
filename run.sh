#!/bin/sh

cat bin/osmium.bin - | tools/bin/emu bootloader/bin/bootloader.bin $@
