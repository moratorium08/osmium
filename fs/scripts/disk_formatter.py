import os
import argparse
import struct
import logging

def p32(val):
    struct.pack('<I', val)

def p16(val):
    struct.pack('<H', val)

def p8(val):
    struct.pack('<B', val)

def st_mode2mode(m):
    return 0

# type
DIRECTORY_TYPE = 1
FILE_TYPE = 2

class Data:
    def write(self, image):
        raise NotImplemented

class SuperBlock(Block):
    pass


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
class Directory(Block):
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
                indices.append(data.write(p))

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

class File(Block):
    def __init__(self, path):
        self.name = os.path.basename(path)


def data_factory(path):
    if os.path.isfile(path):
        return File(path)
    elif os.path.isdir(path):
        return Directory(path)
    else:
        logging.warning('%s is ignored because this is not a file nor a directory', path)
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

N_BLOCKS = 100000
BLOCK_SIZE = 4096
class Image:
    def __init__(root, disk, n_blocks=N_BLOCKS):
        if type(root) != Directory:
            raise TypeError("root must be a directory")
        self.root = root
        self.disk = open(disk, 'rb+')
        self.n_blocks = n_blocks

        padding = (N_BLOCKS % (BLOCK_SIZE * 8)) != 0
        blocks_base_index = N_BLOCKS // (BLOCK_SIZE * 8) + padding + 1 # super block
        # free area stack. for debugging simplicity, use from smaller index
        self.management = list(range(N_BLOCKS - 1, blocks_base_index - 1, -1))

    def _alloc_block(self):
        return self.management.pop()

    def _free_block(self, index):
        return self.management.append(index)

    def _write_block_at(self, data, index):
        place = index * BLOCK_SIZE
        self.disk.seek(place)
        self.disk.write(data)

    def write_block(data):
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
            raise Exception('block size is too large: {} > {}'.format(len(data), BLOCK_SIZE))

        index = self._alloc_block()
        self._write_block_at(data, index)

        return index

    def _write_management(self):
        pass

    def _write_super_block(self):
        pass

    def _finalize(self):
        self._write_management()
        self._write__super_block()

    def generate(self, outchan):
        self.root.write(self)
        self._finalize()

def main():
    argparse.ArgumentParser(description='instant disk image formatter')
    parser.add_argument('dir', help='source directory')
    parser.add_argument('image', help='image filename')
    args = parser.parse_args()


if __name__ == '__main__':
    main()
