#![no_std]
#[macro_use]
extern crate bitflags;
extern crate rlibc;

pub mod dir;
pub mod regular;
pub mod hardware;

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

trait BlockManager {
}


#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Id(u32);

impl Id {
    pub fn is_super(self) -> bool {
        self.0 == 0
    }
}

struct File {
    Regular(regular::Regular)
}

struct<'a> FileSystem<'a> {
    root: dir::Directory<'a>,
}

impl FileSystem {
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
                    return Ok(DataWrapper::File(FileWrapper::from(f, *id)));
                }
                Err(e) => return Err(e),
            }
        }
        Err(FileError::NotFound)
    }

    pub fn search(&mut self, name: [u8; 256]) -> Result<File, FileError> {
        let root_id = self.super_block.root_directory_index;

        let root_data = self.get(root_id)?;
        let dir: &mut Directory;
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
}
