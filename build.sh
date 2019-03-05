#!/bin/sh

# できればcargoで完結させたいが、一旦めんどくさいので
cd misc
./build.sh
cd ..

cd bootloader
./build.sh
cd ..

cd kernel
./build.sh
cd ..
