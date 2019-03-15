pub struct MemoryBlockManager<'a> {
    data: &'a mut [u8; BLOCKSIZE * N_BLOCKS],
    super_block: &'a mut SuperBlock,
}

impl<'a> MemoryBlockManager<'a> {
    pub fn new(data: &'a mut [u8; BLOCKSIZE * N_BLOCKS]) -> BlockManager<'a> {
        let super_block = unsafe { &mut *(data.as_mut_ptr() as *mut SuperBlock) };
        BlockManager { data, super_block }
    }

    pub fn alloc_block(&mut self) -> Result<Id, FileError> {
        for k in 0..self.super_block.n_blocks {
            let id = Id(k);
            if !self.is_valid(id) {
                self.mark_as_used(id);
                let p = unsafe {&mut *(&mut self.data
                    [BLOCKSIZE * k]
                    as *mut u8 as usize
                    as *mut [u8; BLOCKSIZE])};
                rlibc::memset(p.as_mut_ptr(), 0, BLOCKSIZE);
                return Ok(Id(k))
            }
        }
        Err(FileError::NoSpace)
    }

    pub fn free_block(&mut self, index: Id) -> Result<(), FileError> {
        BlockManager::check_is_not_super(id)?;
        self.valid_or_err()?;
        self.mark_as_unused(index);
        Ok(())
    }

    pub fn read_block(&mut self, block: &mut [u8; BLOCKSIZE]) -> Result<(), FileError> {
        BlockManager::check_is_not_super(id)?;
        self.valid_or_err()?;
        for i in 0..BLOCKSIZE {
            block[i] = self.data[id.0 as usize * BLOCKSIZE + i];
        }
        Ok(())
    }

    fn valid_or_err(&self, id: Id) -> Result<(), FileError> {
        if self.is_valid(id) {
            Ok(()) 
        } else {
            Err(FileError::InternalError)
        }
    }

    pub fn write_data(&mut self, data: &[u8], offset: u32, size: u32) -> Result<(), FileError> {
        BlockManager::check_is_not_super(id)?;
        self.valid_or_err()?;
        for i in 0..size {
            self.data[id.0 as usize * BLOCKSIZE + offset as usize + i as usize] = data[i as usize];
        }
    }

    fn check_is_not_super(id: Id) -> Result<(), FileError> {
        if id.is_super() {
            Err(FileError::InternalError)
        } else {
            Ok(())
        }
    }

    pub fn is_valid(&self, id: Id) -> bool {
        let k = id.0;
        let i = k / 8;
        let j = k % 8;
        let index = BLOCKSIZE + i;
        let x = (self.data[index] >> j) & 1;
        x == 0
    }

    pub fn mark_as_used(&mut self, id: Id) {
        let k = id.0;
        let i = k / 8;
        let j = k % 8;
        let index = BLOCKSIZE + i;
        let x = (self.data[index] >> j) & 1;
        self.data[index] |= 1 << j;
    }

    pub fn mark_as_unused(&mut self, id: Id) {
        let k = id.0;
        let i = k / 8;
        let j = k % 8;
        let index = BLOCKSIZE + i;
        let x = (self.data[index] >> j) & 1;
        self.data[index] &= ~(1 << j);
    }
}