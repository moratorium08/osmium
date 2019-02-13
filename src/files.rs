/* simulates file systems */
/* This is very poor system in order to create shell */
use core::slice;

const DIRECTORY_FILE_LIMIT: usize = 4;

extern "C" {
    static nop_start: u8;
    static nop_end: u8;
}

#[derive(Clone, Copy)]
struct MemoryFile {
    pub name: &'static str,
    pub start: &'static u8,
    pub end: &'static u8,
}

impl MemoryFile {
    pub fn size(&self) -> usize {
        let start = (self.start as *const u8) as usize;
        let end = (self.end as *const u8) as usize;
        end - start
    }
}

struct MemoryDirectory {
    pub name: &'static str,
    pub files: [Option<MemoryFile>; DIRECTORY_FILE_LIMIT],
}

static ROOT: MemoryDirectory = MemoryDirectory {
    name: "",
    files: [
        Some(MemoryFile {
            name: "nop",
            start: unsafe { &nop_start },
            end: unsafe { &nop_end },
        }),
        None,
        None,
        None,
    ],
};

impl MemoryDirectory {
    pub fn search(&self, name: &str) -> Option<MemoryFile> {
        for i in 0..DIRECTORY_FILE_LIMIT {
            match self.files[i] {
                Some(file) if file.name == name => return Some(file),
                _ => (),
            }
        }
        None
    }
}

pub struct File<'a> {
    name: &'a str,
    bytes: *const [u8],
    size: usize,
}

impl<'a> File<'a> {
    pub fn new(name: &'a str, bytes: *const [u8], size: usize) -> File<'a> {
        File { name, bytes, size }
    }
}

pub fn search<'a>(filename: &'a str) -> Option<File<'a>> {
    match ROOT.search(filename) {
        Some(file) => {
            let size = file.size();
            let bytes: &[u8] = unsafe { slice::from_raw_parts(file.start as *const u8, size) };
            Some(File::new(filename, bytes, size))
        }
        None => None,
    }
}
