use array_init::array_init;
use core::ops;
use csr;

const MEM_SIZE: usize = 2 * (1 << 9);
pub const LOG_PGSIZE: usize = 12;
pub const PGSIZE: usize = 1 << LOG_PGSIZE;
pub const N_FRAMES: usize = MEM_SIZE / PGSIZE;
pub const PAGE_ENTRY_SIZE: usize = 4;
pub const N_PAGE_ENTRY: usize = PGSIZE / PAGE_ENTRY_SIZE;
const RECURSIVE_ENTRY: usize = N_PAGE_ENTRY - 2;

const KERN_END: usize = 0x80000; // TODO: use linker to specify where it should be

fn start_paging() {
    csr::SATP::enable_paging();
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(u64);

impl PhysAddr {
    pub fn new(addr: u32) -> PhysAddr {
        PhysAddr(0)
    }
    pub fn from_page_index(i: usize) -> PhysAddr {
        let i = i as u64;
        let size = PGSIZE as u64;
        PhysAddr(i * size)
    }
    pub fn start_addr(&self) -> u64 {
        self.0 & 0x3fffff000 // 34 bit
    }
    pub fn floor_pgsize(self) -> PhysAddr {
        PhysAddr(self.start_addr())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(u32);

impl VirtAddr {
    pub fn new(addr: u32) -> VirtAddr {
        VirtAddr(addr)
    }
    pub fn page_start_addr(&self) -> u32 {
        self.0 & 0xfffff000
    }

    pub fn as_mut_ptr<T>(self) -> *mut T {
        let addr = self.0;
        let addr: *mut T = addr as *mut T;
        addr
    }
}

bitflags! {
    pub struct Flag: u32{
        const VALID = 1 << 0;
        const READ  = 1 << 1;
        const WRITE = 1 << 2;
        const EXEC  = 1 << 3;
        const USER  = 1 << 4;
    }
}

#[derive(Clone, Copy)]
pub struct Page {
    ref_count: u32,
    addr: VirtAddr,
}

impl Page {
    pub fn from_addr(addr: VirtAddr) -> Page {
        Page { ref_count: 0, addr }
    }
    pub fn base_addr(&self) -> VirtAddr {
        self.addr
    }
    pub fn vpn1(&self) -> u32 {
        self.addr.page_start_addr() >> 22
    }
    pub fn vpn0(&self) -> u32 {
        (self.addr.page_start_addr() >> 12) & 0x3ff
    }
    pub fn vpns(&self) -> [u32; 2] {
        let vpn1 = self.vpn1();
        let vpn0 = self.vpn0();
        [vpn0, vpn1]
    }
    pub fn from_vpns(vpns: [u32; 2]) -> Page {
        let addr = VirtAddr::new((vpns[1] << 22) | (vpns[0] << 12));
        Page { ref_count: 0, addr }
    }
}

#[derive(Clone, Copy)]
pub struct Frame {
    addr: PhysAddr,
}

impl Frame {
    fn void_frame() -> Frame {
        Frame { addr: PhysAddr(0) }
    }

    pub fn from_addr(addr: PhysAddr) -> Frame {
        Frame { addr }
    }

    pub fn to_ppn(&self) -> u32 {
        /* 34 -> 32(22) bits */
        (self.addr.floor_pgsize().0 >> 2) as u32
    }
}

// Fixed size memory allocator by 'stack-like' simple data structure
pub struct Allocator {
    frames: [Frame; N_FRAMES],
    stack: usize,
}

pub enum PageError {
    FailedToAllocMemory,
    ProgramError(&'static str),
    MapError,
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
    pub fn alloc(&mut self) -> Result<Frame, PageError> {
        if self.stack == 0 {
            Err(PageError::FailedToAllocMemory)
        } else {
            self.stack -= 1;
            Ok(self.frames[self.stack].clone())
        }
    }

    pub fn dealloc(&mut self, frame: Frame) -> Result<(), PageError> {
        if self.stack == N_FRAMES {
            Err(PageError::ProgramError("frame stack overflow"))
        } else {
            self.frames[self.stack] = frame;
            self.stack += 1;
            Ok(())
        }
    }
}

#[derive(Copy, Clone)]
pub struct PageTableEntry {
    entry: u32,
}

impl PageTableEntry {
    fn zero() -> PageTableEntry {
        PageTableEntry { entry: 0 }
    }
    fn flag(&self) -> Flag {
        Flag::from_bits_truncate(self.entry)
    }
    fn is_valid(&self) -> bool {
        self.flag().contains(Flag::VALID)
    }
    fn set_frame(&mut self, frame: Frame, flag: Flag) {
        self.entry = frame.to_ppn() | flag.bits()
    }
}

#[repr(align(4096))]
#[repr(C)]
pub struct PageTable {
    entries: [PageTableEntry; N_PAGE_ENTRY],
}

impl PageTable {
    fn init(&mut self) {
        for i in 0..N_PAGE_ENTRY {
            self.entries[i] = PageTableEntry::zero();
        }
    }
    pub unsafe fn gen_recursive<'a>(base: *mut u32) -> &'a mut PageTable {
        let mut table = &mut *(base as *mut PageTable);

        let table_ptr = &mut table.entries[0] as *mut PageTableEntry;
        let table_ptr = table_ptr as u64;

        let frame = Frame::from_addr(PhysAddr(table_ptr));

        table.entries[RECURSIVE_ENTRY].set_frame(frame, Flag::WRITE | Flag::READ | Flag::VALID);

        table
    }
}

impl ops::Index<usize> for PageTable {
    type Output = PageTableEntry;
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl ops::IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

pub struct Map<'a> {
    dir: &'a mut PageTable,
}

impl<'a> Map<'a> {
    pub fn new(dir: &'a mut PageTable) -> Map {
        Map { dir }
    }

    pub fn create_next_table(
        entry: &'a mut PageTableEntry,
        next_table_page: Page,
        allocator: &mut Allocator,
    ) -> Result<&'a mut PageTable, PageError> {
        let initialize = if !entry.is_valid() {
            let frame = allocator.alloc()?;
            entry.set_frame(frame, Flag::VALID | Flag::READ | Flag::WRITE);
            true
        } else {
            false
        };

        let ptr: *mut PageTable = next_table_page.base_addr().as_mut_ptr();
        let table: &mut PageTable = unsafe { &mut (*ptr) };

        if initialize {
            table.init();
        }
        Ok(table)
    }

    pub fn map(
        &mut self,
        page: Page,
        frame: Frame,
        flag: Flag,
        allocator: &mut Allocator,
    ) -> Result<(), PageError> {
        let vpn1_page = vpn1_page(page);
        let vpn1 =
            Map::create_next_table(&mut self.dir[page.vpn1() as usize], vpn1_page, allocator)?;
        let entry = &mut vpn1[page.vpn0() as usize];

        if entry.is_valid() {
            return Err(PageError::ProgramError("tried to map already mapped page"));
        }
        entry.set_frame(frame, flag);

        Ok(())
    }

    pub fn map_region(
        &mut self,
        virt_addr: VirtAddr,
        phys_addr: PhysAddr,
        size: usize,
        flag: Flag,
    ) -> Result<(), PageError> {
        Ok(())
    }
}

fn vpn1_page(page: Page) -> Page {
    Page::from_vpns([page.vpn1(), RECURSIVE_ENTRY as u32])
}
