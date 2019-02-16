use core::fmt;
use core::ops;
use csr::satp;
use csr::{CSRRead, CSRWrite};

pub const LOG_PGSIZE: usize = 12;
pub const PGSIZE: usize = 1 << LOG_PGSIZE;
const MEM_SIZE: usize = (1 << 31) + PGSIZE; // memory is 2GB, IO is last frame
pub const N_FRAMES: usize = MEM_SIZE / PGSIZE;
pub const PAGE_ENTRY_SIZE: usize = 4;
pub const N_PAGE_ENTRY: usize = PGSIZE / PAGE_ENTRY_SIZE;
pub const TMP_PAGE_ENTRY: usize = N_PAGE_ENTRY - 1;
pub const USER_MEMORY_BASE: usize = 0x80400000;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
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
    pub fn as_mut_kernel_ptr<T>(self) -> *mut T {
        let addr = self.0;
        let addr: *mut T = addr as *mut T;
        addr
    }
    pub fn kern_virt_addr(&self) -> VirtAddr {
        VirtAddr(self.0 as u32)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
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

    pub fn from_ptr<T>(ptr: *const T) -> VirtAddr {
        let addr = ptr as u32;
        VirtAddr(addr)
    }

    pub fn to_u32(&self) -> u32 {
        self.0
    }

    pub fn offset(&self, offset: u32) -> VirtAddr {
        VirtAddr(offset + self.0)
    }

    pub fn kern_phys_addr(&self) -> PhysAddr {
        PhysAddr(self.0 as u64)
    }
}

bitflags! {
    pub struct Flag: u32{
        const VALID = 1 << 0;
        const READ  = 1 << 1;
        const WRITE = 1 << 2;
        const EXEC  = 1 << 3;
        const USER  = 1 << 4;
        const COW   = 1 << 8;
    }
}

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
pub struct Frame {
    addr: PhysAddr,
}

impl Frame {
    pub fn from_addr(addr: PhysAddr) -> Frame {
        Frame { addr }
    }

    pub fn to_ppn(&self) -> u32 {
        /* 34 -> 32(22) bits */
        (self.addr.floor_pgsize().0 >> 2) as u32
    }

    pub fn phys_addr(&self) -> PhysAddr {
        self.addr
    }
}

// Fixed size memory allocator by 'stack-like' simple data structure
pub struct Allocator<'a> {
    frames: &'a mut [Frame; N_FRAMES],
    stack: usize,
}

#[derive(Copy, Clone, Debug)]
pub enum PageError {
    FailedToAllocMemory,
    ProgramError(&'static str),
    MapError,
    AlreadyMapped,
    IllegalAddress,
}

impl<'a> Allocator<'a> {
    pub unsafe fn new(frames: *mut u32, is_used: &Fn(usize) -> bool) -> Allocator<'a> {
        let frames = &mut *(frames as *mut [Frame; N_FRAMES]);
        let mut stack = 0;
        for i in 0..N_FRAMES {
            if is_used(i * PGSIZE) {
                continue;
            }
            frames[stack] = Frame::from_addr(PhysAddr::from_page_index(i));
            if i % 100000 == 99999 {
                println!("{} % completed", (100 * i) / N_FRAMES);
            }
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
        //println!("{:x} | {:x} = {:x}", frame.to_ppn(), flag.bits(), frame.to_ppn() | flag.bits());
        self.entry = frame.to_ppn() | flag.bits()
    }
    fn phys_addr(&self) -> PhysAddr {
        PhysAddr(((self.entry & 0xfffffc00) as u64) << 2)
    }
    fn frame(&self) -> Frame {
        Frame::from_addr(self.phys_addr())
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
    pub fn setup_tmp_table(table: &mut PageTable, tmp_table: &mut PageTable) {
        table[TMP_PAGE_ENTRY].set_frame(
            Frame::from_addr(PhysAddr((tmp_table as *const PageTable) as u64)),
            Flag::VALID,
        );
        tmp_table[TMP_PAGE_ENTRY].set_frame(
            Frame::from_addr(PhysAddr((table as *const PageTable) as u64)),
            Flag::VALID,
        );
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
    tmp_page: &'a mut PageTable,
}

impl<'a> Map<'a> {
    pub fn new(dir: &'a mut PageTable, tmp_page: &'a mut PageTable) -> Map<'a> {
        Map { dir, tmp_page }
    }

    pub fn clone_dir(&self, map: &mut Map) {
        for i in 0..(N_PAGE_ENTRY - 1) {
            map.dir[i] = self.dir[i];
        }
    }

    pub fn create_cow_user_memory(
        &self,
        map: &mut Map,
        allocator: &mut Allocator,
    ) -> Result<(), PageError> {
        // satp on self's
        let old_satp = satp::SATP::read();
        satp::SATP::set_ppn(self.ppn());

        let user_entry = USER_MEMORY_BASE / (PGSIZE * N_PAGE_ENTRY);
        for i in user_entry..(N_PAGE_ENTRY - 1) {
            let flag = self.dir[i].flag();
            if flag.contains(Flag::VALID) {
                // create page_table
                let table = Map::get_vpn1_page_table(i);
                for j in 0..(N_PAGE_ENTRY - 1) {
                    // a process which forked two times but still contains COW has !VALID & COW
                    if table[j].flag().contains(Flag::VALID | Flag::COW) {
                        let new_flag: Flag;
                        if table[j].flag().contains(Flag::WRITE) {
                            new_flag = table[j].flag() & (!Flag::WRITE) | Flag::COW;
                        } else {
                            new_flag = table[j].flag();
                        }
                        let page = Page::from_vpns([j as u32, i as u32]);

                        let old_satp = satp::SATP::read();
                        satp::SATP::set_ppn(map.ppn());
                        map.map(page, table[j].frame(), new_flag, allocator)?;
                        old_satp.commit()
                    }
                }
            }
        }
        old_satp.commit();
        Ok(())
    }

    fn vpn1_page(page: Page) -> Page {
        Page::from_vpns([page.vpn1(), TMP_PAGE_ENTRY as u32])
    }

    fn get_vpn1_page_table<'b>(index: usize) -> &'b mut PageTable {
        assert!(index < N_PAGE_ENTRY);
        let page = Page::from_vpns([index as u32, TMP_PAGE_ENTRY as u32]);
        let ptr = page.base_addr().as_mut_ptr();
        unsafe { &mut (*ptr) }
    }

    fn create_next_table(
        &mut self,
        page: Page,
        allocator: &mut Allocator,
        boot: bool,
    ) -> Result<&'a mut PageTable, PageError> {
        let frame: Frame;
        let initialize;
        {
            let entry = &mut self.dir[page.vpn1() as usize];
            let tmp_entry = &mut self.tmp_page[page.vpn1() as usize];
            initialize = if !entry.is_valid() {
                frame = allocator.alloc()?;
                entry.set_frame(frame, Flag::VALID);
                tmp_entry.set_frame(frame, Flag::READ | Flag::WRITE | Flag::VALID);
                println!("entry {:x}", tmp_entry.flag());
                true
            } else {
                frame = Frame::from_addr(entry.phys_addr());
                false
            };
        }

        let ptr: *mut PageTable;
        if boot {
            ptr = frame.phys_addr().as_mut_kernel_ptr();
        } else {
            ptr = Map::vpn1_page(page).base_addr().as_mut_ptr();
        }
        let table: &mut PageTable = unsafe { &mut (*ptr) };
        if initialize {
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
        let vpn1 = self.create_next_table(page, allocator, boot)?;
        let entry = &mut vpn1[page.vpn0() as usize];

        /*if entry.is_valid() {
            return Err(PageError::ProgramError("tried to map already mapped page"));
        }*/
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

    pub fn identity_map(
        &mut self,
        frame: Frame,
        flag: Flag,
        allocator: &mut Allocator,
    ) -> Result<(), PageError> {
        let phys_addr = frame.phys_addr().to_u64();
        if phys_addr >= (1 << 32) {
            return Err(PageError::IllegalAddress);
        }
        let virt_addr = VirtAddr::new(phys_addr as u32);
        let page = Page::from_addr(virt_addr);
        self.map(page, frame, flag, allocator)
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

    pub fn ppn(&self) -> u32 {
        println!("page dir {:x}", self.dir as *const PageTable as u32);
        (VirtAddr::from_ptr(self.dir as *const PageTable)
            .kern_phys_addr()
            .to_u64()
            >> LOG_PGSIZE) as u32
    }
}
