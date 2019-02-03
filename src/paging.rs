use array_init::array_init;
use csr;

const MEM_SIZE: usize = 2 * (1 << 9);
const PGSIZE: usize = 4096;
const N_FRAMES: usize = MEM_SIZE / PGSIZE;
const PAGE_ENTRY_SIZE: usize = 4;
const PAGE_ENTRY: usize = PGSIZE / PAGE_ENTRY_SIZE;
const RECURSIVE_ENTRY: usize = PAGE_ENTRY - 2;

const KERN_END: usize = 0x80000; // TODO: use linker to specify where it should be

fn start_paging() {
    csr::SATP::enable_paging();
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(u64);

impl PhysAddr {
    pub fn from_page_index(i: usize) -> PhysAddr {
        let i = i as u64;
        let size = PGSIZE as u64;
        PhysAddr(i * size)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(u32);

bitflags! {
    struct Flag: u32{
        const VALID = 1 << 0;
        const READ  = 1 << 1;
        const WRITE = 1 << 2;
        const EXEC  = 1 << 3;
        const USER  = 1 << 4;
    }
}

struct Page {
    ref_count: u32,
    addr: VirtAddr,
}

impl Page {}

struct Frame {
    addr: PhysAddr,
}

impl Frame {
    fn void_frame() -> Frame {
        Frame { addr: PhysAddr(0) }
    }

    pub fn from_addr(addr: PhysAddr) -> Frame {
        Frame { addr }
    }
}

// Fixed size memory allocator by 'stack-like' simple data structure
struct Allocator {
    frames: [Frame; N_FRAMES],
    stack: usize,
}

impl Allocator {
    pub fn new() -> Allocator {
        let mut frames: [Frame; N_FRAMES] = array_init(|_| Frame::void_frame());
        let mut stack = 0;
        // TODO: create memory map
        for i in 0..N_FRAMES {
            if i < KERN_END / PGSIZE + 1 {
                continue;
            }
            frames[stack] = Frame::from_addr(PhysAddr::from_page_index(i));
            stack += 1;
        }
        Allocator { frames, stack }
    }
    pub fn alloc() {}

    pub fn dealloc() {}
}

impl Page {}

#[derive(Clone)]
pub struct PageTableEntry {
    entry: u32,
}

#[repr(align(4096))]
#[repr(C)]
pub struct PageTable {
    entries: [PageTableEntry; PAGE_ENTRY_SIZE],
}

struct Directory<'a> {
    dir: &'a mut PageTable,
}

impl<'a> Directory<'a> {
    pub fn new(dir: &'a mut PageTable) -> Directory {
        Directory { dir }
    }

    pub fn map(&mut self, page: Page, frame: Frame, flags: Flag, allocator: &mut Allocator) {}
}
