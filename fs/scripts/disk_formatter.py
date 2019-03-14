#!/usr/bin/env python3
import os
import argparse
import struct
import logging

N_BLOCKS = 20  # 00
BLOCK_SIZE = 4096


def p32(val):
    return struct.pack('<I', val)


def p16(val):
    return struct.pack('<H', val)


def p8(val):
    return struct.pack('<B', val)


def st_mode2mode(m):
    return 0


def read_exact(f, size):
    cur = 0
    s = b''
    while True:
        tmp = f.read(size - cur)
        cur += len(tmp)
        if len(tmp) == 0:
            break
        s += tmp
        if cur == size:
            break
    return s


# type
DIRECTORY_TYPE = 1
FILE_TYPE = 2


class Data:
    def write(self, image):
        raise NotImplemented


'''
Directory data
--------------
type: u8
dummy: u8
name: char[256]. ends with \x00 if length is less than 256
permission: u16
owner: u16
group: u16
size: u32
----------
'indirect' block indices[957]
----------
direcotry cannot hold more than 957 * 1024 files
'''

DIRECTORY_FILE_MAX = 957 * (BLOCK_SIZE // 4)


class Directory(Data):
    def __init__(self, path):
        self.name = os.path.basename(path)
        # should implement
        self.permission = 0
        self.owner = 0
        self.group = 0
        self.path = path

    def _write(self, image):
        block = b''
        block += p8(DIRECTORY_TYPE)
        block += p8(0)
        block += self.name.encode('utf-8')
        block += p16(self.permission)
        block += p16(self.owner)
        block += p16(self.group)
        block += p32(self.size)
        for index in self.indirect_blocks:
            block += p32(index)
        return image.write_block(block)

    def write(self, image):
        # 1. write files to image and save these indices
        files = os.listdir(self.path)
        if len(files) > DIRECTORY_FILE_MAX:
            raise Exception('{} holds too much files: {} > {}',
                            self.path,
                            len(files),
                            DIRECTORY_FILE_MAX)
        indices = []
        for name in files:
            p = os.path.join(self.path, name)
            data = data_factory(p)
            if data is not None:
                indices.append(data.write(image))

        # 2. write the indirect indices to image and save these indices
        indirect_blocks = []
        count = 0
        tmp = b''
        for index in indices:
            if count == (BLOCK_SIZE // 4):
                indirect_blocks.append(image.write_block(tmp))
                tmp = b''
                count = 0
            tmp += p32(index)
        if count != 0:
            indirect_blocks.append(image.write_block(tmp))

        # 3. finally write self to image
        self.size = len(indices)
        self.indirect_blocks = indirect_blocks
        return self._write(image)


class File(Data):
    def __init__(self, path):
        self.name = os.path.basename(path)
        # should implement
        self.permission = 0
        self.owner = 0
        self.group = 0
        self.path = path

    def _write(self, image):
        block = b''
        block += p8(FILE_TYPE)
        block += p8(0)
        block += self.name.encode('utf-8')
        block += p16(self.permission)
        block += p16(self.owner)
        block += p16(self.group)
        block += p32(self.size)
        for index in self.indirect_blocks:
            block += p32(index)
        return image.write_block(block)

    def write(self, image):
        with open(self.path, "rb") as f:
            size = 0
            blocks = []
            while True:
                s = read_exact(f, BLOCK_SIZE)
                block = image.write_block(s)
                blocks.append(block)
                size += len(s)
                if len(s) < BLOCK_SIZE:
                    break

            indirect_blocks1 = []
            indirect_blocks2 = b''
            direct_block = b''
            for block in blocks:
                if len(indirect_blocks2) == BLOCK_SIZE:
                    b = image.write_block(indirect_blocks2)
                    indirect_blocks1.append(b)
                    indirect_blocks2 = b''
                if len(direct_block) == BLOCK_SIZE:
                    b = image.write_block(direct_block)
                    indirect_blocks2 += p32(b)
                    direct_block = b''
                direct_block += p32(block)

            if len(indirect_blocks2) == BLOCK_SIZE:
                b = image.write_block(indirect_blocks2)
                indirect_blocks1.append(b)
                indirect_blocks2 = b''
            if len(direct_block) == BLOCK_SIZE:
                b = image.write_block(direct_block)
                indirect_blocks2 += p32(b)
                direct_block = b''
            if len(indirect_blocks2) == BLOCK_SIZE:
                b = image.write_block(indirect_blocks2)
                indirect_blocks1.append(b)
                indirect_blocks2 = b''
            if len(direct_block) != 0:
                b = image.write_block(direct_block)
                indirect_blocks2 += p32(b)
            if len(indirect_blocks2) != 0:
                b = image.write_block(indirect_blocks2)
                indirect_blocks1.append(b)

            self.indirect_blocks = indirect_blocks1
            self.size = size
        return self._write(image)


def data_factory(path):
    if os.path.isfile(path):
        return File(path)
    elif os.path.isdir(path):
        return Directory(path)
    else:
        logging.warning(
            '%s is ignored because this is not a file nor a directory', path)
        return None


'''
--------------
| SuperBlock |
--------------
|            |
| Management |
|            |
|------------|
|  Blocks    |
|     .      |
|     .      |
|     .      |

SuperBlock: holds size of blocks, the index where 'Blocks' starts, where root
directory.
Management: holds which 'Block' is used to corresponding bit
Block: File/Directory data

* filename convention: must end with '\x00'
'''


def fill_zero(f, size):
    # batch by BLOCK_SIZE
    s = b'\x00' * BLOCK_SIZE
    for i in range(size // BLOCK_SIZE):
        f.write(s)
    f.write(b'\x00' * (size % BLOCK_SIZE))
    f.seek(0)


class Image:

    def __init__(self, root, disk, n_blocks=N_BLOCKS):
        if type(root) != Directory:
            raise TypeError("root must be a directory")
        self.root = root
        self.disk = open(disk, 'wb+')
        self.n_blocks = n_blocks
        padding = (N_BLOCKS % (BLOCK_SIZE * 8)) != 0
        # super block
        blocks_base_index = N_BLOCKS // (BLOCK_SIZE * 8) + padding + 1
        # free area stack. for debugging simplicity, use from smaller index
        self.management = list(range(N_BLOCKS - 1, blocks_base_index - 1, -1))

        # first initialize disk file
        fill_zero(self.disk, (blocks_base_index + N_BLOCKS) * BLOCK_SIZE)

    def _alloc_block(self):
        return self.management.pop()

    def _free_block(self, index):
        return self.management.append(index)

    def _write_block_at(self, data, index):
        place = index * BLOCK_SIZE
        self.disk.seek(place)
        self.disk.write(data)

    def write_block(self, data):
        '''
        write block to disk

        Parameters
        ----------
        data: bytes

        Returns
        -------
        block_index: int
          where the data has been written
        '''
        if len(data) > BLOCK_SIZE:
            raise Exception('block size is too large: {} > {}'.format(
                len(data), BLOCK_SIZE))

        index = self._alloc_block()
        self._write_block_at(data, index)

        return index

    def _write_management(self):
        pass

    def _write_super_block(self):
        pass

    def _finalize(self):
        self._write_management()
        self._write_super_block()

    def generate(self):
        self.root.write(self)
        self._finalize()


def main():
    parser = argparse.ArgumentParser(
        description='instant disk image formatter')
    parser.add_argument('root', help='source directory')
    parser.add_argument('image', help='image filename')
    args = parser.parse_args()

    root = Directory(args.root)
    image = Image(root, args.image)
    image.generate()


if __name__ == '__main__':
    main()
