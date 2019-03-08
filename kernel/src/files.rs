/* simulates file systems */
/* This is very poor system in order to create shell */
use core::slice;

const DIRECTORY_FILE_LIMIT: usize = 200;

extern "C" {
    static nop_start: u8;
    static nop_end: u8;
    static loop_start: u8;
    static loop_end: u8;
    static loopback_start: u8;
    static loopback_end: u8;
    static syscaller_start: u8;
    static syscaller_end: u8;
    static hello_start: u8;
    static hello_end: u8;
    static sh_start: u8;
    static sh_end: u8;
    static tic_start: u8;
    static tic_end: u8;
    static catch_ball_start: u8;
    static catch_ball_end: u8;
/*
static ls_start: u8;
static ls_end: u8;
*/
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

static mut ROOT: MemoryDirectory = MemoryDirectory {
    name: "",
    files: [None; DIRECTORY_FILE_LIMIT],
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

pub fn init() {
    unsafe {
        let l = [
            ("/bin/nop", &nop_start, &nop_end),
            ("/bin/loop", &loop_start, &loop_end),
            ("/bin/syscaller", &syscaller_start, &syscaller_end),
            ("/bin/hello", &hello_start, &hello_end),
            ("/bin/sh", &sh_start, &sh_end),
            ("/bin/tic", &tic_start, &tic_end),
            ("/bin/catch_ball", &catch_ball_start, &catch_ball_end),
        ];
        for (i, (n, s, e)) in l.iter().enumerate() {
            ROOT.files[i] = Some(MemoryFile {
                name: n,
                start: unsafe { &s },
                end: unsafe { &e },
            });
        }
    }
}

pub struct File<'a> {
    pub name: &'a str,
    pub bytes: *const [u8],
    pub size: usize,
}

impl<'a> File<'a> {
    pub fn new(name: &'a str, bytes: *const [u8], size: usize) -> File<'a> {
        File { name, bytes, size }
    }
}

pub fn search<'a>(filename: &'a str) -> Option<File<'a>> {
    match unsafe { ROOT.search(filename) } {
        Some(file) => {
            let size = file.size();
            let bytes: &[u8] = unsafe { slice::from_raw_parts(file.start as *const u8, size) };
            Some(File::new(filename, bytes, size))
        }
        None => None,
    }
}
