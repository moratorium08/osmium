#!/bin/sh

rm -rf tools
git clone --recursive https://github.com/cpu-3/tools
make build -C tools
