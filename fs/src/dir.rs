use core::slice;
use crate::{Flag, Id, TypeRepr, N_POINTER_PER_DIR, BLOCKSIZE};
use crate::regular;

#[repr(C)]
pub struct DirectoryRaw {
    ty: TypeRepr,
    name: [u8; 256],
    permission: Flag,
    dummy2: u8,
    dummy3: u8,
    file_count: u32,
    files: [Id; N_POINTER_PER_DIR],
}

pub struct Directory{
    index: Id,
}

/*
impl Directory {
    pub fn new(
        bm: &'a mut BlockManager,
        name: [u8; 256],
    ) -> Result<DirectoryWrapper<'a>, FileError> {
        match bm.alloc_block() {
            Some((index, dir)) => {
                let d = Directory::from_bytes(dir);
                d.name = name;
                d.file_count = 0;
                Ok(DirectoryWrapper { index, dir: d })
            }
            None => return Err(FileError::NoSpace),
        }
    }
    pub fn add_file(&mut self, fw: &'a regular::Regular) -> Result<(), FileError> {
        if self.dir.file_count >= N_POINTER_PER_DIR as u32 {
            return Err(FileError::NoSpace);
        }

        self.dir.files[self.dir.file_count as usize] = fw.index;
        self.dir.file_count += 1;
        Ok(())
    }

    pub fn list_files(&self) -> slice::Iter<Id> {
        (&self.dir.files).iter()
    }

    pub fn from(dir: &'a mut Directory, index: Id) -> DirectoryWrapper<'a> {
        DirectoryWrapper { dir, index }
    }
}
*/