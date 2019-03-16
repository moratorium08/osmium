use crate::*;
use core::slice;

pub fn round_up(x: u32, modulo: u32) -> u32 {
    let tmp = x % modulo;
    let y = modulo - if tmp == 0 { modulo } else { tmp };
    x + y
}

#[test]
fn test_round_up() {
    assert_eq!(round_up(3000, 4096), 4096);
    assert_eq!(round_up(0, 4096), 0);
    assert_eq!(round_up(4096, 4096), 4096);
    assert_eq!(round_up(4096, 4096), 4096);
}

#[repr(C)]
pub struct RegularRaw {
    ty: u8,
    dummy1: u8,
    pub name: [u8; 256],
    pub permission: u16,
    pub owner: u16,
    pub group: u16,
    size: u32,
    data: [u32; N_POINTER_PER_FILE],
}

pub struct Regular {
    pub id: Id,
    pointer: u32,
}

fn as_table_mut<'a>(block: &'a mut Block) -> &'a mut [u32] {
    let p = block.as_mut_ptr() as *mut u32;
    unsafe { slice::from_raw_parts_mut(p, BLOCKSIZE / 4) }
}

struct Index(usize, usize);

impl Index {
    fn from_block_id(id: usize) -> Index {
        Index(id / (BLOCKSIZE / 4), id % (BLOCKSIZE / 4))
    }
    fn from_pointer(p: u32) -> Index {
        let id = p as usize / BLOCKSIZE;
        Index::from_block_id(id)
    }
}

impl Regular {
    fn get_meta_block<'a>(&self, bm: &mut BlockManager) -> Result<&'a mut RegularRaw, FileError> {
        let block = &mut bm.read_block(self.id)?;
        Ok(unsafe { &mut *(block.as_mut_ptr() as *mut RegularRaw) })
    }
    fn write_meta_block(
        &self,
        bm: &mut BlockManager,
        meta_block: &RegularRaw,
    ) -> Result<(), FileError> {
        let meta_block = unsafe {
            &*(slice::from_raw_parts((meta_block as *const RegularRaw) as *const u8, BLOCKSIZE))
        };
        // hmm.. this translation should be removed.
        let mut fixed = [0u8; BLOCKSIZE];
        for i in 0..BLOCKSIZE {
            fixed[i] = meta_block[i];
        }
        bm.write_block(self.id, fixed)
    }
    fn get_table_entry(&self, bm: &mut BlockManager) -> Result<u32, FileError> {
        let meta_block = self.get_meta_block(bm)?;
        let index = self.current_index();
        let id = meta_block.data[index.0];
        let mut block = bm.read_block(Id(id))?;

        let table = as_table_mut(&mut block);
        Ok(table[index.1])
    }
    fn get_current_block_id(&self, bm: &mut BlockManager) -> Result<Id, FileError> {
        Ok(Id(self.get_table_entry(bm)?))
    }
    fn get_current_block(&self, bm: &mut BlockManager) -> Result<Block, FileError> {
        let id = self.get_current_block_id(bm)?;
        bm.read_block(id)
    }

    fn write_current_block(&self, bm: &mut BlockManager, block: Block) -> Result<(), FileError> {
        let id = self.get_current_block_id(bm)?;
        bm.write_block(id, block)
    }
    pub fn create(
        bm: &mut BlockManager,
        name: [u8; 256],
        permission: Flag,
    ) -> Result<Regular, FileError> {
        let id = bm.alloc_block()?;
        let regular = Regular { id, pointer: 0 };
        let meta_block = regular.get_meta_block(bm)?;
        meta_block.name = name;
        meta_block.permission = permission.bits();
        meta_block.ty = Type::File.to_repr();
        meta_block.size = 0;
        // TBD: owner/ group
        Ok(regular)
    }
    fn current_offset(&self) -> usize {
        self.pointer as usize % BLOCKSIZE
    }
    fn current_index(&self) -> Index {
        Index::from_pointer(self.pointer)
    }
    fn validate_current_pointer_write(&self) -> Result<(), FileError> {
        if self.pointer >= (BLOCKSIZE * N_POINTER_PER_FILE) as u32 {
            return Err(FileError::TooLarge);
        }
        Ok(())
    }

    fn alloc_block(&self, bm: &mut BlockManager, index: Index) -> Result<(), FileError> {
        let allocated = bm.alloc_block()?;
        let meta_block = self.get_meta_block(bm)?;
        let id = meta_block.data[index.0];
        let mut block = bm.read_block(Id(id))?;
        let table = as_table_mut(&mut block);
        table[index.1] = allocated.0;
        Ok(())
    }

    pub fn write(
        &mut self,
        bm: &mut BlockManager,
        data: &[u8],
        size: usize,
    ) -> Result<(), FileError> {
        let meta_block = self.get_meta_block(bm)?;
        let mut max_writable = round_up(meta_block.size, BLOCKSIZE as u32);

        let mut offset = self.current_offset();
        let mut written = 0;
        while written < size {
            if self.pointer >= max_writable {
                self.validate_current_pointer_write()?;
                self.alloc_block(bm, Index::from_pointer(max_writable))?;
                max_writable += BLOCKSIZE as u32;
            }
            let mut block = self.get_current_block(bm)?;
            while written < size && offset < BLOCKSIZE {
                block[offset] = data[written];
                offset += 1;
                written += 1;
                self.pointer += 1;
            }
            self.write_current_block(bm, block)?;
            offset %= BLOCKSIZE;
        }
        Ok(())
    }

    pub fn read(
        &mut self,
        bm: &mut BlockManager,
        data: &mut [u8],
        size: usize,
    ) -> Result<(), FileError> {
        let meta_block = self.get_meta_block(bm)?;
        let max_readable = meta_block.size;

        let mut offset = self.current_offset();
        let mut written = 0;
        while written < size {
            let block = self.get_current_block(bm)?;
            while written < size && offset < BLOCKSIZE {
                if self.pointer >= max_readable {
                    return Err(FileError::EndOfFile);
                }
                data[written] = block[offset];
                // a little verbose (for efficiency)
                offset += 1;
                written += 1;
                self.pointer += 1;
            }
            offset %= BLOCKSIZE;
        }
        self.write_meta_block(bm, meta_block)?;
        Ok(())
    }

    pub fn seek(&mut self, bm: &mut BlockManager, offset: i32) -> Result<(), FileError> {
        let new_ptr = (self.pointer as i32) + offset;

        if new_ptr < 0 || new_ptr as usize > BLOCKSIZE * N_POINTER_PER_FILE {
            Err(FileError::InvalidOffset)
        } else {
            let meta_block = self.get_meta_block(bm)?;

            for i in 0..((new_ptr as usize) / BLOCKSIZE) {
                if meta_block.data[i] == 0 {
                    self.alloc_block(bm, Index::from_block_id(i))?;
                }
            }
            self.pointer = new_ptr as u32;
            Ok(())
        }
    }
}
