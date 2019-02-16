import os
import binascii

BIN = "bin/osmium.bin"

def main():
    with open(BIN, "rb") as f:
        s = f.read()

    l = hex(len(s))[2:]
    if len(l) % 2 == 1:
        l = '0' + l
    length = binascii.unhexlify(l)
    if len(length) > 4:
        print("OS is too big.")
        return

    pad = b'\x00' * (4 - len(length))
    with open(BIN + ".lengthed", "wb") as f:
        f.write(pad + length)
        f.write(s)
main()
