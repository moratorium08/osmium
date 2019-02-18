use core::slice;

const BLOCKSIZE: usize = 4096;
const N_BLOCKS: usize = 100000;

bitflags! {
    pub struct Flag: u8 {
        const USER_READ = 1 << 0;
        const USER_WRITE  = 1 << 1;
        const USER_EXEC = 1 << 2;
        const GROUP_READ  = 1 << 3;
        const GROUP_WRITE  = 1 << 4;
        const GROUP_EXEC   = 1 << 8;
    }
}

trait Block: Sized {
    fn bytes(&self) -> &[u8];
    fn from_bytes(&self, &mut [u8]) -> &mut Self;
}

#[repr(C)]
pub struct File {
    name: [u8; 256],
    authority: Flag,
    dummy1: u8,
    dummy2: u8,
    dummy3: u8,
    size: u32,
    data: [u32; (BLOCKSIZE / 4) - 256 / 4 - 1 - 1],
}

#[repr(C)]
pub struct Directory {
    name: [u8; 256],
    authority: Flag,
    dummy1: u8,
    dummy2: u8,
    dummy3: u8,
    file_count: u32,
    files: [u32; (BLOCKSIZE / 4) - 256 / 4 - 1 - 1],
}

/*
--------------
| SuperBlock |
--------------
|            |
| Management |
|------------|
|  Blocks    |
|     .      |
|     .      |
|     .      |

*/

#[repr(C)]
pub struct SuperBlock {
    root_directory_index: u32,
    management_index: u32,
    mangement_size: u32,
    dummy: [u32; BLOCKSIZE / 4 - 3],
}

impl Block for File {
    fn bytes(&self) -> &[u8] {
        slice::from_raw_parts(self as *const File as *const u8, BLOCKSIZE)
    }
    fn from_bytes(&self, bytes: &mut [u8]) -> &mut File {
        unsafe { &mut *(bytes.as_mut_ptr() as *mut File) }
    }
}

impl Block for Directory {
    fn bytes(&self) -> &[u8] {
        slice::from_raw_parts(self as *const Directory as *const u8, BLOCKSIZE)
    }
    fn from_bytes(&self, bytes: &mut [u8]) -> &mut Directory {
        unsafe { &mut *(bytes.as_mut_ptr() as *mut Directory) }
    }
}

pub struct BlockManager<'a> {
    data: &'a mut [u8; BLOCKSIZE * N_BLOCKS],
    super_block: &'a mut SuperBlock,
}

pub struct Id(u32);

impl<'a> BlockManager<'a> {
    pub fn new(data: &'a mut [u8; BLOCKSIZE * N_BLOCKS]) -> BlockManager<'a> {
        let super_block = unsafe { &mut *(data.as_mut_ptr() as *mut SuperBlock) };
        BlockManager { data, super_block }
    }

    pub fn alloc_block(&mut self) -> Option<&'a mut [u8; BLOCKSIZE]> {
        for k in 0..self.super_block.mangement_size {
            let i = k / 8;
            let j = k % 8;
            let index = BLOCKSIZE + i as usize * BLOCKSIZE;
            let x = (self.data[index] >> j) & 1;
            if x == 0 {
                self.data[index] |= 1 << j;
                return Some(unsafe {
                    &mut *(&mut self.data
                        [BLOCKSIZE + self.super_block.management_index as usize / 8 + index]
                        as *mut u8 as usize as *mut [u8; BLOCKSIZE])
                });
            }
        }
        None
    }
    // TODO: dealloc
}
