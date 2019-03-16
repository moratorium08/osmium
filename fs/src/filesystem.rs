use crate::*;

pub struct FileSystem<'a> {
    root: dir::Directory,
    block_manager: BlockManager<'a>,
}

impl <'a>FileSystem<'a> {
    pub fn search_inner(
        &mut self,
        path: PathObject,
        current_dir: dir::Directory,
    ) -> Result<File, FileError> {

        match current_dir.search(&mut self.block_manager, path)? {
            Some((id, path)) if path.is_end() => {
                File::from_id(&mut self.block_manager, id)
            },
            Some((id, path)) => {
                self.search_inner(path, dir::Directory::new(id))
            }
            None => {
                Err(FileError::NotFound)
            }
        }

    }

    pub fn search(&mut self, name: &[u8]) -> Result<File, FileError> {
        if name[0] != b'/' {
            return Err(FileError::IllegalPath);
        }

        let mut path = PathObject::new(&name);
        path.countup(1); // first slash

        self.search_inner(path, self.root.clone());

        Err(FileError::NotFound)
    }
}