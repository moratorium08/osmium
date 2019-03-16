use crate::*;

pub struct MemoryBlockManager<'a> {
    data: &'a mut [u8; BLOCKSIZE * N_BLOCKS],
}

impl<'a> MemoryBlockManager<'a> {
    pub fn new(data: &'a mut [u8; BLOCKSIZE * N_BLOCKS]) -> MemoryBlockManager<'a> {
        MemoryBlockManager { data }
    }
}

impl<'a> BlockManager<'a> for MemoryBlockManager<'a> {
    fn super_block(&mut self) -> &'a mut SuperBlock {
        unsafe { &mut *(self.data.as_mut_ptr() as *mut SuperBlock) }
    }

    fn fill_block(&mut self, id: Id, val: u8) -> Result<(), FileError> {
        let p = unsafe {
            &mut *(&mut self.data[BLOCKSIZE * id.0 as usize] as *mut u8 as usize
                as *mut [u8; BLOCKSIZE])
        };
        unsafe { rlibc::memset(p.as_mut_ptr(), val as i32, BLOCKSIZE) };
        Ok(())
    }

    fn read_block(&mut self, id: Id) -> Result<Block, FileError> {
        id.check_is_not_super()?;
        self.valid_or_err(id)?;
        let mut block = [0u8; BLOCKSIZE];
        for i in 0..BLOCKSIZE {
            block[i] = self.data[id.0 as usize * BLOCKSIZE + i];
        }
        Ok(block)
    }

    fn write_data(&mut self, id: Id, data: &[u8], offset: u32, size: u32) -> Result<(), FileError> {
        id.check_is_not_super()?;
        self.valid_or_err(id)?;
        for i in 0..size {
            self.data[id.0 as usize * BLOCKSIZE + offset as usize + i as usize] = data[i as usize];
        }
        Ok(())
    }

    fn is_valid(&self, id: Id) -> bool {
        let k = id.0;
        let i = k / 8;
        let j = k % 8;
        let index = BLOCKSIZE + i as usize;
        let x = (self.data[index] >> j) & 1;
        x == 0
    }

    fn mark_as_used(&mut self, id: Id) {
        let k = id.0;
        let i = k / 8;
        let j = k % 8;
        let index = BLOCKSIZE + i as usize;
        self.data[index] |= 1 << j;
    }

    fn mark_as_unused(&mut self, id: Id) {
        let k = id.0;
        let i = k / 8;
        let j = k % 8;
        let index = BLOCKSIZE + i as usize;
        self.data[index] &= !(1 << j);
    }
}
