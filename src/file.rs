use core::slice;
use memutil;

const BLOCKSIZE: usize = 4096;
const N_BLOCKS: usize = 100000;
const N_POINTER_PER_DIR: usize = (BLOCKSIZE / 4) - 256 / 4 - 1 - 1;
const N_POINTER_PER_FILE: usize = (BLOCKSIZE / 4) - 256 / 4 - 1 - 1;

pub enum FileError {
    NoSpace,
    InvalidOffset,
    TooLarge,
    EndOfFile,
    InternalError,
}
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
    fn from_bytes(&mut [u8]) -> &mut Self;
}

#[repr(C)]
pub struct File {
    name: [u8; 256],
    authority: Flag,
    dummy1: u8,
    dummy2: u8,
    dummy3: u8,
    size: u32,
    data: [Id; N_POINTER_PER_FILE],
}

#[repr(C)]
pub struct Directory {
    name: [u8; 256],
    authority: Flag,
    dummy1: u8,
    dummy2: u8,
    dummy3: u8,
    file_count: u32,
    files: [u32; N_POINTER_PER_DIR],
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
    fn from_bytes(bytes: &mut [u8]) -> &mut File {
        unsafe { &mut *(bytes.as_mut_ptr() as *mut File) }
    }
}

struct FileWrapper<'a> {
    index: Id,
    file: &'a mut File,
    pointer: u32,
}

impl<'a> FileWrapper<'a> {
    fn new(bm: &'a mut BlockManager) -> Result<FileWrapper<'a>, FileError> {
        match bm.alloc_block() {
            Some((index, file)) => Ok(FileWrapper {
                index,
                file: File::from_bytes(file),
                pointer: 0,
            }),
            None => return Err(FileError::NoSpace),
        }
    }
    fn current_offset(&self) -> usize {
        self.pointer as usize % BLOCKSIZE
    }
    fn current_index(&self) -> usize {
        self.pointer as usize / BLOCKSIZE
    }
    fn validate_current_pointer_write(&self) -> Result<(), FileError> {
        if self.pointer >= (BLOCKSIZE * N_POINTER_PER_FILE) as u32 {
            return Err(FileError::TooLarge);
        }
        Ok(())
    }
    fn validate_current_pointer_read(&self) -> Result<(), FileError> {
        if self.pointer >= self.file.size {
            return Err(FileError::EndOfFile);
        }
        Ok(())
    }
    fn read_byte(&mut self, byte: u8, manager: &mut BlockManager) -> Result<u8, FileError> {
        self.validate_current_pointer_read()?;
        let offset = self.current_offset();
        let ptr_index = self.current_index();

        let id = self.file.data[ptr_index];
        manager.read_byte(id, offset)
    }
    fn write_byte(&mut self, byte: u8, manager: &mut BlockManager) -> Result<(), FileError> {
        if self.pointer >= (BLOCKSIZE * N_POINTER_PER_FILE) as u32 {
            return Err(FileError::TooLarge);
        }
        let offset = self.current_offset();
        let ptr_index = self.current_index();
        if offset as u32 == 0 {
            match manager.alloc_block() {
                Some((i, f)) => self.file.data[ptr_index] = i,
                None => return Err(FileError::NoSpace),
            }
        }
        let id = self.file.data[ptr_index];
        self.pointer += 1;
        self.file.size = u32::max(self.pointer, self.file.size);
        manager.write_byte(id, offset, byte)
    }
    fn write(&mut self, data: &[u8], size: usize) -> Result<(), FileError> {
        for i in 0..size {}
        Ok(())
    }
    fn read(&mut self, data: &mut [u8], size: usize) -> Result<(), FileError> {
        Ok(())
    }
    fn seek(&mut self, seek: i32, manager: &mut BlockManager) -> Result<(), FileError> {
        let new_ptr = (self.pointer as i32) + seek;

        if new_ptr < 0 || new_ptr as usize > BLOCKSIZE * N_POINTER_PER_FILE {
            Err(FileError::InvalidOffset)
        } else {
            for i in 0..N_POINTER_PER_FILE {
                if self.file.data[i].is_super() {
                    match manager.alloc_block() {
                        Some((id, _)) => {
                            self.file.data[i] = id;
                        }
                        None => return Err(FileError::NoSpace),
                    }
                }
            }
            self.pointer = new_ptr as u32;
            Ok(())
        }
    }
}

impl Block for Directory {
    fn bytes(&self) -> &[u8] {
        slice::from_raw_parts(self as *const Directory as *const u8, BLOCKSIZE)
    }
    fn from_bytes(bytes: &mut [u8]) -> &mut Directory {
        unsafe { &mut *(bytes.as_mut_ptr() as *mut Directory) }
    }
}

pub struct BlockManager<'a> {
    data: &'a mut [u8; BLOCKSIZE * N_BLOCKS],
    super_block: &'a mut SuperBlock,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Id(u32);

impl Id {
    pub fn is_super(self) -> bool {
        self.0 == 0
    }
}

impl<'a> BlockManager<'a> {
    pub fn new(data: &'a mut [u8; BLOCKSIZE * N_BLOCKS]) -> BlockManager<'a> {
        let super_block = unsafe { &mut *(data.as_mut_ptr() as *mut SuperBlock) };
        BlockManager { data, super_block }
    }

    pub fn alloc_block(&mut self) -> Option<(Id, &'a mut [u8; BLOCKSIZE])> {
        for k in 0..self.super_block.mangement_size {
            let i = k / 8;
            let j = k % 8;
            let index = BLOCKSIZE + i as usize * BLOCKSIZE;
            let x = (self.data[index] >> j) & 1;
            if x == 0 {
                self.data[index] |= 1 << j;
                return Some((Id(index as u32), unsafe {
                    let p = &mut *(&mut self.data
                        [BLOCKSIZE + self.super_block.management_index as usize / 8 + index]
                        as *mut u8 as usize
                        as *mut [u8; BLOCKSIZE]);
                    memutil::memset(p.as_mut_ptr(), 0, BLOCKSIZE);
                    p
                }));
            }
        }
        None
    }

    fn check_is_not_super(id: Id) -> Result<(), FileError> {
        if id.is_super() {
            Err(FileError::InternalError)
        } else {
            Ok(())
        }
    }

    pub fn write_byte(&mut self, id: Id, offset: usize, byte: u8) -> Result<(), FileError> {
        BlockManager::check_is_not_super(id)?;
        self.data[id.0 as usize * BLOCKSIZE + offset] = byte;
        Ok(())
    }
    pub fn read_byte(&mut self, id: Id, offset: usize) -> Result<u8, FileError> {
        BlockManager::check_is_not_super(id)?;
        Ok(self.data[id.0 as usize * BLOCKSIZE + offset])
    }
    // TODO: dealloc
}
