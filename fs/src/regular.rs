use crate::FileError;

#[repr(C)]
pub struct RegularRaw {
    ty: TypeRepr,
    name: [u8; 256],
    authority: Flag,
    dummy2: u8,
    dummy3: u8,
    size: u32,
    data: [Id; N_POINTER_PER_FILE],
}

impl RegularRaw {
    fn bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self as *const File as *const u8, BLOCKSIZE) }
    }
    fn from_bytes(bytes: &mut [u8]) -> &mut File {
        unsafe { &mut *(bytes.as_mut_ptr() as *mut File) }
    }
}

pub struct Regular<'a> {
    pub index: Id,
    pointer: u32,
}

impl<'a> Regular<'a> {
    fn new(bm: &'a mut BlockManager, name: [u8; 256]) -> Result<Regular<'a>, FileError> {
        match bm.alloc_block() {
            Some((index, file)) => {
                let file = File::from_bytes(file);
                file.name = name;
                Ok(FileWrapper {
                    index,
                    file,
                    pointer: 0,
                })
            }
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
    pub fn write(&mut self, data: &[u8], size: usize) -> Result<(), FileError> {
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
    fn from(file: &'a mut RegularRaw, index: Id) -> Regular{
        FileWrapper {
            file,
            index,
            pointer: 0,
        }
    }
}
