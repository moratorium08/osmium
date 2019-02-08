use array_init::array_init;
use core::fmt;
use core::ops;
use csr::satp;

pub const LOG_PGSIZE: usize = 12;
pub const PGSIZE: usize = 1 << LOG_PGSIZE;
const MEM_SIZE: usize = (1 << 31) + PGSIZE; // memory is 2GB, IO is last frame
pub const N_FRAMES: usize = MEM_SIZE / PGSIZE;
pub const PAGE_ENTRY_SIZE: usize = 4;
pub const N_PAGE_ENTRY: usize = PGSIZE / PAGE_ENTRY_SIZE;
const RECURSIVE_ENTRY: usize = N_PAGE_ENTRY - 2;

const KERN_END: usize = 0x80000; // TODO: use linker to specify where it should be

fn start_paging() {
    satp::SATP::enable_paging();
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(u64);

impl fmt::Display for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PhysAddr({})", self.0)
    }
}

impl PhysAddr {
    pub fn new(addr: u64) -> PhysAddr {
        PhysAddr(addr)
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
    pub fn to_u64(&self) -> u64 {
        self.0
    }
    pub fn offset(&self, offset: u64) -> PhysAddr {
        PhysAddr(offset + self.0)
    }
    // only in boot process
    fn as_mut_ptr<T>(self) -> *mut T {
        let addr = self.0;
        let addr: *mut T = addr as *mut T;
        addr
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(u32);

impl fmt::Display for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VirtAddr({})", self.0)
    }
}

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

    pub fn to_u32(&self) -> u32 {
        self.0
    }

    pub fn offset(&self, offset: u32) -> VirtAddr {
        VirtAddr(offset + self.0)
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
        Page {
            ref_count: 0,
            addr: VirtAddr::new(addr.page_start_addr()),
        }
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
    pub fn range(addr: VirtAddr, size: u32) -> PageRange {
        let end = addr.page_start_addr() + size;
        let md = end % (PGSIZE as u32);
        let pad = if md == 0 { 0 } else { (PGSIZE as u32) - md };
        PageRange {
            start: Page::from_addr(addr),
            end: Page::from_addr(VirtAddr::new(addr.page_start_addr() + size + pad)),
        }
    }
    pub fn next_page(&self) -> Page {
        Page {
            ref_count: 0,
            addr: VirtAddr::new(self.addr.page_start_addr() + (PGSIZE as u32)),
        }
    }
}

// 半開区間[start, end)
pub struct PageRange {
    start: Page,
    end: Page,
}

impl Iterator for PageRange {
    type Item = Page;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start.base_addr() < self.end.base_addr() {
            let page = self.start.clone();
            self.start = self.start.next_page();
            Some(page)
        } else {
            None
        }
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
        //(self.addr.floor_pgsize().0 >> 2) as u32
        self.addr.floor_pgsize().0 as u32
    }

    fn phys_addr(&self) -> PhysAddr {
        self.addr
    }
}

// Fixed size memory allocator by 'stack-like' simple data structure
pub struct Allocator<'a> {
    frames: &'a mut [Frame; N_FRAMES],
    stack: usize,
}

#[derive(Debug)]
pub enum PageError {
    FailedToAllocMemory,
    ProgramError(&'static str),
    MapError,
}

impl<'a> Allocator<'a> {
    pub unsafe fn new(frames: *mut u32, is_used: &Fn(usize) -> bool) -> Allocator<'a> {
        let frames = &mut *(frames as *mut [Frame; N_FRAMES]);
        let mut stack = 0;
        for i in 0..N_FRAMES {
            if i % 10000 == 9999 {
                println!("{} % completed", (100 * i) / N_FRAMES);
            }
            if is_used(i * PGSIZE) {
                continue;
            }
            frames[stack] = Frame::from_addr(PhysAddr::from_page_index(i));
            stack += 1;
        }
        println!("N_FRAMES: {}, stack: {}", N_FRAMES, stack);
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
    fn phys_addr(&self) -> PhysAddr {
        PhysAddr((self.entry & 0xfffff000) as u64)
    }
}

#[repr(align(4096))]
#[repr(C)]
pub struct PageTable {
    entries: [PageTableEntry; N_PAGE_ENTRY],
}

impl PageTable {
    fn init(&mut self) {
        println!("page table placement {:p}", self);
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

    fn create_next_table(
        entry: &'a mut PageTableEntry,
        next_table_page: Page,
        allocator: &mut Allocator,
        boot: bool,
    ) -> Result<&'a mut PageTable, PageError> {
        let frame: Frame;
        let initialize = if !entry.is_valid() {
            frame = allocator.alloc()?;
            entry.set_frame(frame, Flag::VALID);
            true
        } else {
            frame = Frame::from_addr(entry.phys_addr());
            false
        };

        let table: &mut PageTable;
        if boot {
            let ptr: *mut PageTable = frame.phys_addr().as_mut_ptr();
            table = unsafe { &mut (*ptr) };
        } else {
            let ptr: *mut PageTable = next_table_page.base_addr().as_mut_ptr();
            table = unsafe { &mut (*ptr) };
        }

        if initialize {
            println!("init");
            table.init();
        }
        Ok(table)
    }
    fn map_inner(
        &mut self,
        page: Page,
        frame: Frame,
        flag: Flag,
        allocator: &mut Allocator,
        boot: bool,
    ) -> Result<(), PageError> {
        let vpn1_page = vpn1_page(page);
        let vpn1 = Map::create_next_table(
            &mut self.dir[page.vpn1() as usize],
            vpn1_page,
            allocator,
            boot,
        )?;
        println!(
            "{}: {} -> {}",
            page.vpn0(),
            page.base_addr(),
            frame.phys_addr()
        );
        let entry = &mut vpn1[page.vpn0() as usize];

        if entry.is_valid() {
            return Err(PageError::ProgramError("tried to map already mapped page"));
        }
        entry.set_frame(frame, flag);

        Ok(())
    }

    pub fn boot_map(
        &mut self,
        page: Page,
        frame: Frame,
        flag: Flag,
        allocator: &mut Allocator,
    ) -> Result<(), PageError> {
        self.map_inner(page, frame, flag, allocator, true)
    }

    pub fn map(
        &mut self,
        page: Page,
        frame: Frame,
        flag: Flag,
        allocator: &mut Allocator,
    ) -> Result<(), PageError> {
        self.map_inner(page, frame, flag, allocator, false)
    }

    fn map_region_inner(
        &mut self,
        virt_addr: VirtAddr,
        phys_addr: PhysAddr,
        size: usize,
        flag: Flag,
        allocator: &mut Allocator,
        boot: bool,
    ) -> Result<(), PageError> {
        if virt_addr.to_u32() % (PGSIZE as u32) != 0 {
            return Err(PageError::ProgramError("page alignment is invalid"));
        }
        if phys_addr.to_u64() % (PGSIZE as u64) != 0 {
            return Err(PageError::ProgramError("page alignment is invalid"));
        }
        let tmp = size % PGSIZE;
        let pad = if tmp == 0 { 0 } else { PGSIZE - tmp };
        let n_pages = (size + pad) / PGSIZE;
        println!("{} / {} = {}", size, PGSIZE, n_pages);
        for i in 0..n_pages {
            self.map_inner(
                Page::from_addr(virt_addr.offset((i * PGSIZE) as u32)),
                Frame::from_addr(phys_addr.offset((i * PGSIZE) as u64)),
                flag,
                allocator,
                boot,
            )?;
        }
        Ok(())
    }

    pub fn map_region(
        &mut self,
        virt_addr: VirtAddr,
        phys_addr: PhysAddr,
        size: usize,
        flag: Flag,
        allocator: &mut Allocator,
    ) -> Result<(), PageError> {
        self.map_region_inner(virt_addr, phys_addr, size, flag, allocator, false)
    }

    // after boot, create identity map of kernel properties
    pub fn boot_map_region(
        &mut self,
        virt_addr: VirtAddr,
        phys_addr: PhysAddr,
        size: usize,
        flag: Flag,
        allocator: &mut Allocator,
    ) -> Result<(), PageError> {
        self.map_region_inner(virt_addr, phys_addr, size, flag, allocator, true)
    }
}

fn vpn1_page(page: Page) -> Page {
    Page::from_vpns([page.vpn1(), RECURSIVE_ENTRY as u32])
}
