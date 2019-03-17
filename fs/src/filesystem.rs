use crate::*;

pub struct FileSystem<'a> {
    root: dir::Directory,
    block_manager: &'a mut BlockManager<'a>,
}

impl<'a> FileSystem<'a> {
    pub fn new(block_manager: &'a mut BlockManager<'a>) -> FileSystem<'a> {
        let super_block = block_manager.super_block();
        let root = dir::Directory::new(Id(super_block.root_directory_index));
        FileSystem {
            root,
            block_manager,
        }
    }

    pub fn search_inner(
        &mut self,
        path: PathObject,
        current_dir: dir::Directory,
    ) -> Result<File, FileError> {
        match current_dir.search(self.block_manager, path)? {
            Some((id, path)) if path.is_end() => File::from_id(self.block_manager, id),
            Some((id, path)) => self.search_inner(path, dir::Directory::new(id)),
            None => Err(FileError::NotFound),
        }
    }

    pub fn search(&mut self, name: &[u8]) -> Result<File, FileError> {
        if name[0] != b'/' {
            return Err(FileError::IllegalPath);
        }

        let mut path = PathObject::new(&name);
        path.countup(1); // first slash

        self.search_inner(path, self.root.clone())
    }

    pub fn read(&mut self, file: &mut File, buf: &mut [u8], size: usize) -> Result<(), FileError> {
        match file {
            File::Regular(ref mut r) => r.read(self.block_manager, buf, size),
            _ => Err(FileError::IllegalType),
        }
    }

    pub fn write(&mut self, file: &mut File, buf: &[u8], size: usize) -> Result<(), FileError> {
        match file {
            File::Regular(ref mut r) => r.write(self.block_manager, buf, size),
            _ => Err(FileError::IllegalType),
        }
    }

    pub fn seek(&mut self, file: &mut File, offset: i32) -> Result<(), FileError> {
        match file {
            File::Regular(ref mut r) => r.seek(self.block_manager, offset),
            _ => Err(FileError::IllegalType),
        }
    }
}
