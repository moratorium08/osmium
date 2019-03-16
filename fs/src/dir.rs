use crate::*;
use core::slice;

#[repr(C)]
pub struct DirectoryRaw {
    ty: TypeRepr,
    dummy: u8,
    name: [u8; 256],
    permission: u16,
    pub owner: u16,
    pub group: u16,
    file_count: u32,
    files: [u32; N_POINTER_PER_DIR],
}

pub struct Directory {
    id: Id,
}

impl FileLike for Directory {
    type Raw = DirectoryRaw;

    fn my_id(&self) -> Id {
        self.id
    }
}

#[derive(Copy, Clone)]
struct Index(usize, usize);

impl Index {
    fn from_block_id(id: usize) -> Index {
        Index(id / (BLOCKSIZE / 4), id % (BLOCKSIZE / 4))
    }
    fn to_block_id(&self) -> usize {
        self.0 * (BLOCKSIZE / 4) + self.1
    }
    fn from_pointer(p: u32) -> Index {
        let id = p as usize / BLOCKSIZE;
        Index::from_block_id(id)
    }
}

impl Directory {
    pub fn create(
        bm: &mut BlockManager,
        name: [u8; 256],
        permission: Flag,
    ) -> Result<Directory, FileError> {
        let id = bm.alloc_block()?;
        let directory = Directory { id };
        let meta_block = directory.get_meta_block(bm)?;
        meta_block.name = name;
        meta_block.permission = permission.bits();
        meta_block.ty = Type::File.to_repr();
        meta_block.file_count = 0;
        directory.write_meta_block(bm, meta_block)?;
        // TBD: owner/ group
        Ok(directory)
    }

    fn find_free_block(&self, bm: &mut BlockManager) -> Result<Index, FileError> {
        let meta_block = self.get_meta_block(bm)?;
        for i in 0..N_POINTER_PER_DIR {
            let id = meta_block.files[i];
            if id == 0 {
                let id = bm.alloc_block()?;
                meta_block.files[i] = id.0;
                self.write_meta_block(bm, meta_block)?;
                return Ok(Index(i, 0));
            }
            let mut block = bm.read_block(Id(id))?;
            let table = as_table_mut(&mut block);
            for j in 0..(BLOCKSIZE / 4) {
                if table[j] == 0 {
                    return Ok(Index(i, j));
                }
            }
        }
        Err(FileError::NoSpace)
    }

    fn find_id(&self, bm: &mut BlockManager, id: Id) -> Result<Index, FileError> {
        let meta_block = self.get_meta_block(bm)?;
        for i in 0..N_POINTER_PER_DIR {
            let index = meta_block.files[i];
            if index == 0 {
                continue;
            }
            let mut block = bm.read_block(Id(index))?;
            let table = as_table_mut(&mut block);
            for j in 0..(BLOCKSIZE / 4) {
                if table[j] == id.0 {
                    return Ok(Index(i, j));
                }
            }
        }
        Err(FileError::NoSpace)
    }

    fn find_next_block(
        &self,
        bm: &mut BlockManager,
        index: Index,
    ) -> Result<Option<(Index, Id)>, FileError> {
        let meta_block = self.get_meta_block(bm)?;
        for i in index.0..N_POINTER_PER_DIR {
            let id = meta_block.files[i];
            if id == 0 {
                continue;
            }
            let mut block = bm.read_block(Id(id))?;
            let table = as_table_mut(&mut block);
            let starting_point = if index.0 == i { index.1 } else { 0 };
            for j in starting_point..(BLOCKSIZE / 4) {
                if table[j] != 0 {
                    return Ok(Some((Index(i, j), Id(table[j]))));
                }
            }
        }
        Ok(None)
    }

    fn write_entry(&self, bm: &mut BlockManager, entry: Index, id: Id) -> Result<(), FileError> {
        let meta_block = self.get_meta_block(bm)?;
        let index = meta_block.files[entry.0];
        let mut block = bm.read_block(Id(index))?;
        let table = as_table_mut(&mut block);
        table[entry.1] = id.0;
        bm.write_block(Id(index), block)?;
        Ok(())
    }

    fn remove_entry(&self, bm: &mut BlockManager, entry: Index) -> Result<(), FileError> {
        self.write_entry(bm, entry, Id(0))
    }

    pub fn add_file(&mut self, bm: &mut BlockManager, id: Id) -> Result<(), FileError> {
        let meta_block = self.get_meta_block(bm)?;
        let entry = self.find_free_block(bm)?;

        self.write_entry(bm, entry, id)?;

        meta_block.file_count += 1;
        self.write_meta_block(bm, meta_block)?;
        Ok(())
    }

    pub fn remove_file(&mut self, bm: &mut BlockManager, id: Id) -> Result<(), FileError> {
        let meta_block = self.get_meta_block(bm)?;
        let index = self.find_id(bm, id)?;
        self.remove_entry(bm, index)?;
        self.write_meta_block(bm, meta_block)?;
        Ok(())
    }

    pub fn get_entry(
        &self,
        bm: &mut BlockManager,
        index: usize,
    ) -> Result<Option<(usize, Id)>, FileError> {
        match self.find_next_block(bm, Index::from_block_id(index))? {
            Some((index, id)) => Ok(Some((index.to_block_id(), id))),
            None => Ok(None),
        }
    }

    pub fn get_n_entry(&self, bm: &mut BlockManager, n: usize) -> Result<Option<Id>, FileError> {
        let mut index = Index::from_block_id(0);
        let mut cnt: usize = 0;
        loop {
            match self.find_next_block(bm, index)? {
                Some((i, _)) if cnt < n => {
                    index = i;
                    cnt += 1;
                }, 
                Some((_, id)) => {
                    return Ok(Some(id));
                },
                None => {
                    return Ok(None);
                }
            }
        }

    }

    pub fn search(
        &self,
        bm: &mut BlockManager,
        path: PathObject,
    ) -> Result<(Id, PathObject), FileError> {
        unimplemented!();
    }
}
