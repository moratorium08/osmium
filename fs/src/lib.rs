#![no_std]
#![feature(core_intrinsics)]
#![feature(bind_by_move_pattern_guards)]
#[macro_use]
extern crate bitflags;
extern crate rlibc;

pub mod dir;
pub mod hardware;
pub mod regular;
pub mod filesystem;

use core::slice;
use core::ops;

const BLOCKSIZE: usize = 4096;
const N_BLOCKS: usize = 100000;
const N_POINTER_PER_DIR: usize = (BLOCKSIZE / 4) - 256 / 4 - 1 - 1 - 1;
const N_POINTER_PER_FILE: usize = (BLOCKSIZE / 4) - 256 / 4 - 1 - 1 - 1;
const N_DUMMIES: usize = (BLOCKSIZE / 4) - 256 / 4 - 1 - 1;

pub enum FileError {
    NoSpace,
    InvalidOffset,
    TooLarge,
    EndOfFile,
    InternalError,
    BrokenFile,
    BrokenFileSystem,
    NotFound,
    IllegalPath,
}

bitflags! {
    pub struct Flag: u16 {
        const USER_READ = 1 << 0;
        const USER_WRITE  = 1 << 1;
        const USER_EXEC = 1 << 2;
        const GROUP_READ  = 1 << 3;
        const GROUP_WRITE  = 1 << 4;
        const GROUP_EXEC   = 1 << 8;
    }
}

type TypeRepr = u8;

type Block = [u8; BLOCKSIZE];

// this should be removed
fn as_table_mut<'a>(block: &'a mut Block) -> &'a mut [u32] {
    let p = block.as_mut_ptr() as *mut u32;
    unsafe { slice::from_raw_parts_mut(p, BLOCKSIZE / 4) }
}

pub enum Type {
    Regular,
    Directory,
}

impl Type {
    fn from_repr(t: TypeRepr) -> Result<Type, FileError> {
        if t == 1 {
            Ok(Type::Regular)
        } else if t == 2 {
            Ok(Type::Directory)
        } else {
            Err(FileError::BrokenFile)
        }
    }
    fn to_repr(self) -> TypeRepr {
        match self {
            Type::Regular => 1,
            Type::Directory => 2,
        }
    }
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

/* filename convention: must end with '\x00' */
*/

#[repr(C)]
pub struct SuperBlock {
    root_directory_index: Id,
    management_index: u32,
    n_blocks: u32,
    block_index: u32,
    dummy: [u32; BLOCKSIZE / 4 - 4],
}

pub trait BlockManager<'a> {
    fn super_block(&mut self) -> &'a mut SuperBlock;
    fn is_valid(&self, id: Id) -> bool;
    fn mark_as_used(&mut self, id: Id);
    fn mark_as_unused(&mut self, id: Id);
    fn fill_block(&mut self, id: Id, val: u8) -> Result<(), FileError>;
    fn read_block(&mut self, id: Id) -> Result<Block, FileError>;
    fn write_data(&mut self, id: Id, data: &[u8], offset: u32, size: u32) -> Result<(), FileError>;

    fn write_block(&mut self, id: Id, block: Block) -> Result<(), FileError> {
        self.write_data(id, &block, 0, BLOCKSIZE as u32)
    }

    fn alloc_block(&mut self) -> Result<Id, FileError> {
        for k in 0..self.super_block().n_blocks {
            let id = Id(k);
            if !self.is_valid(id) {
                self.mark_as_used(id);
                self.fill_block(id, 0)?;
                return Ok(Id(k));
            }
        }
        Err(FileError::NoSpace)
    }

    fn free_block(&mut self, id: Id) -> Result<(), FileError> {
        id.check_is_not_super()?;
        self.valid_or_err(id)?;
        self.mark_as_unused(id);
        Ok(())
    }

    fn valid_or_err(&self, id: Id) -> Result<(), FileError> {
        if self.is_valid(id) {
            Ok(())
        } else {
            Err(FileError::InternalError)
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Id(u32);

impl Id {
    pub fn is_super(self) -> bool {
        self.0 == 0
    }
}

impl Id {
    fn check_is_not_super(&self) -> Result<(), FileError> {
        if self.is_super() {
            Err(FileError::InternalError)
        } else {
            Ok(())
        }
    }
}

pub enum File {
    Regular(regular::Regular),
    Direcotry(dir::Directory),
}

impl File {
    fn from_id(bm: &mut BlockManager, id: Id) -> Result<File, FileError> {
        let file = File::get_file(bm, id)?;
        match Type::from_repr(file.ty)? {
            Type::Regular => Ok(File::Regular(regular::Regular::new(id))),
            Type::Directory => Ok(File::Direcotry(dir::Directory::new(id))),
        }
    }
}

#[repr(C)]
pub struct FileRaw {
    ty: TypeRepr,
    dummy1: u8,
    name: [u8; 256],
    permission: u16,
    pub owner: u16,
    pub group: u16,
    dummy2: [u32; N_DUMMIES],
}

impl FileLike for File {
    type Raw = FileRaw;

    fn my_id(&self) -> Id {
        match self {
            File::Regular(r) => r.my_id(),
            File::Direcotry(d) => d.my_id(),
        }
    }
}

pub trait FileLike {
    type Raw;

    fn my_id(&self) -> Id;

    fn get_file<'a>(bm: &mut BlockManager, id: Id) -> Result<&'a mut Self::Raw, FileError> {
        let block = &mut bm.read_block(id)?;
        Ok(unsafe { &mut *(block.as_mut_ptr() as *mut Self::Raw) })
    } 

    fn get_meta_block<'a>(&self, bm: &mut BlockManager) -> Result<&'a mut Self::Raw, FileError> {
        Self::get_file(bm, self.my_id())
    }
    fn write_meta_block(
        &self,
        bm: &mut BlockManager,
        meta_block: &Self::Raw,
    ) -> Result<(), FileError> {
        let meta_block = unsafe {
            &*(slice::from_raw_parts((meta_block as *const Self::Raw) as *const u8, BLOCKSIZE))
        };
        // hmm.. this translation should be removed.
        let mut fixed = [0u8; BLOCKSIZE];
        for i in 0..BLOCKSIZE {
            fixed[i] = meta_block[i];
        }
        bm.write_block(self.my_id(), fixed)
    }
}

pub struct PathObject<'a> {
    name: &'a [u8],
    current_pos: usize,
}

impl<'a> PathObject<'a> {
    pub fn new(name: &'a [u8]) -> PathObject<'a> {
        PathObject{name, current_pos: 0}
    }
    pub fn countup(&mut self, count: usize) {
        self.current_pos += count;
    }
    pub fn is_end(&self) -> bool {
        self.name[self.current_pos] == 0
    }
}

impl <'a>core::ops::Index<usize> for PathObject<'a> {
    type Output = u8;
    fn index(&self, i: usize) -> &Self::Output {
        &self.name[self.current_pos + i]
    }
}