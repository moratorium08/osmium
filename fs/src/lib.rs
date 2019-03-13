#[macro_use]
extern crate bitflags;
extern crate rlibc;

use core::slice;

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

pub enum Type {
    File,
    Directory,
}

impl Type {
    fn from_repr(t: TypeRepr) -> Result<Type, FileError> {
        if t == 1 {
            Ok(Type::File)
        } else if t == 2 {
            Ok(Type::Directory)
        } else {
            Err(FileError::BrokenFile)
        }
    }
    fn to_repr(self) -> TypeRepr {
        match self {
            Type::File => 1,
            Type::Directory => 2,
        }
    }
}

pub enum Data<'a> {
    File(&'a mut File),
    Directory(&'a mut Directory),
}

impl<'a> Data<'a> {
    pub fn from_ptr(ptr: *mut u8) -> Result<Data<'a>, FileError> {
        let ty = unsafe { *ptr };
        let t = Type::from_repr(ty)?;
        match t {
            Type::File => Ok(Data::File(unsafe { &mut *(ptr as *mut File) })),
            Type::Directory => Ok(Data::Directory(unsafe { &mut *(ptr as *mut Directory) })),
        }
    }
    pub fn name(&'a self) -> &'a [u8; 256] {
        match self {
            Data::File(f) => &f.name,
            Data::Directory(f) => &f.name,
        }
    }
}

pub enum DataWrapper<'a> {
    File(FileWrapper<'a>),
    Directory(DirectoryWrapper<'a>),
}

#[repr(C)]
pub struct File {
    ty: TypeRepr,
    name: [u8; 256],
    authority: Flag,
    dummy2: u8,
    dummy3: u8,
    size: u32,
    data: [Id; N_POINTER_PER_FILE],
}

#[repr(C)]
pub struct Directory {
    ty: TypeRepr,
    name: [u8; 256],
    authority: Flag,
    dummy2: u8,
    dummy3: u8,
    file_count: u32,
    files: [Id; N_POINTER_PER_DIR],
}

pub struct DirectoryWrapper<'a> {
    index: Id,
    dir: &'a mut Directory,
}

impl<'a> DirectoryWrapper<'a> {
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
    pub fn add_file(&mut self, fw: &'a FileWrapper) -> Result<(), FileError> {
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
    mangement_size: u32,
    dummy: [u32; BLOCKSIZE / 4 - 3],
}

impl File {
    fn bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self as *const File as *const u8, BLOCKSIZE) }
    }
    fn from_bytes(bytes: &mut [u8]) -> &mut File {
        unsafe { &mut *(bytes.as_mut_ptr() as *mut File) }
    }
}

pub struct FileWrapper<'a> {
    pub index: Id,
    file: &'a mut File,
    pointer: u32,
}

impl<'a> FileWrapper<'a> {
    fn new(bm: &'a mut BlockManager, name: [u8; 256]) -> Result<FileWrapper<'a>, FileError> {
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
    fn from(file: &'a mut File, index: Id) -> FileWrapper {
        FileWrapper {
            file,
            index,
            pointer: 0,
        }
    }
}

impl Directory {
    pub fn bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self as *const Directory as *const u8, BLOCKSIZE) }
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
                    rlibc::memset(p.as_mut_ptr(), 0, BLOCKSIZE);
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

    pub fn is_valid(&self, id: Id) -> bool {
        let k = id.0;
        let i = k / 8;
        let j = k % 8;
        let index = BLOCKSIZE + i as usize * BLOCKSIZE;
        let x = (self.data[index] >> j) & 1;
        x == 0
    }

    pub fn get(&self, id: Id) -> Result<Data, FileError> {
        if self.is_valid(id) {
            Data::from_ptr(&self.data[id.0 as usize] as *const u8 as *mut u8)
        } else {
            Err(FileError::InternalError)
        }
    }

    pub fn search_inner<'b>(
        &'b self,
        name: &[u8; 256],
        name_idx: usize,
        dir: &'b mut Directory,
    ) -> Result<DataWrapper<'b>, FileError> {
        'outer: for id in dir.files.iter() {
            match self.get(*id) {
                Ok(Data::Directory(d)) => {
                    let mut tmp_idx = name_idx;
                    for i in 0..(256 - name_idx) {
                        if name[name_idx + i] != d.name[i] {
                            // next dir
                            if name[tmp_idx] != b'/' {
                                return self.search_inner(name, tmp_idx, d);
                            }
                            // name is not equal
                            continue 'outer;
                        }
                        if d.name[i] == 0 {
                            break;
                        }
                        tmp_idx += 1;
                    }
                    if name[255] != b'\x00' {
                        return Err(FileError::BrokenFile);
                    }
                    return Ok(DataWrapper::Directory(DirectoryWrapper::from(d, *id)));
                }
                Ok(Data::File(f)) => {
                    let mut tmp_idx = name_idx;
                    for i in 0..(256 - name_idx) {
                        if name[name_idx + i] != f.name[i] {
                            continue 'outer;
                        }
                        if f.name[i] == 0 {
                            break;
                        }
                        tmp_idx += 1;
                    }
                    if name[255] != b'\x00' {
                        return Err(FileError::BrokenFile);
                    }
                    return Ok(DataWrapper::File(FileWrapper::from(f, *id)));
                }
                Err(e) => return Err(e),
            }
        }
        Err(FileError::NotFound)
    }

    pub fn search(&mut self, name: [u8; 256]) -> Result<FileWrapper, FileError> {
        let root_id = self.super_block.root_directory_index;

        let root_data = self.get(root_id)?;
        let mut dir: &mut Directory;
        match root_data {
            Data::Directory(d) => dir = d,
            _ => return Err(FileError::BrokenFileSystem),
        }

        if name[0] != b'/' {
            return Err(FileError::IllegalPath);
        }

        let mut idx = 1;
        self.search_inner(&name, idx, dir);

        Err(FileError::NotFound)
    }
    // TODO: dealloc
}
