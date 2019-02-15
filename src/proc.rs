use core::fmt;
use csr::CSRRead;
use elf;
use memlayout;
use memutil;
use paging;
use satp;
use trap;
use utils;

pub const N_PROCS: usize = 1024;

enum Type {
    User,
}

enum Status {
    Free,
    Running,
    Runnable,
    NotRunnable,
    Zonmie,
}

#[derive(Debug, Copy, Clone)]
pub enum ProcessError {
    FailedToCreateProcess,
    FailedToMap(paging::PageError),
    ProgramError(&'static str),
}

impl ProcessError {
    fn to_str(&self) -> &'static str {
        match self {
            ProcessError::FailedToCreateProcess => "failed to create process",
            ProcessError::FailedToMap(_) => "failed to map",
            ProcessError::ProgramError(s) => s,
        }
    }
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ProcessError({})", self.to_str())
    }
}

#[derive(Copy, Clone)]
pub struct Id(pub u32);

#[repr(C)]
pub struct Process<'a> {
    pub mapper: paging::Map<'a>,
    pub id: Id,
    pub parent_id: Id,
    index: usize,
    proc_type: Type,
    status: Status,
    trap_frame: trap::TrapFrame,
}

impl<'a> Process<'a> {
    pub fn init(&mut self, id: Id, mapper: paging::Map<'a>) {
        self.mapper = mapper;
        self.id = id;
        self.parent_id = id;
        self.proc_type = Type::User;
        self.status = Status::Free;
        self.trap_frame = trap::TrapFrame::new(0, 0);
    }
    // dont touch without ProcessManager
    pub unsafe fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    pub fn size_of() -> usize {
        use core::intrinsics::size_of_val;
        let dummy = 100usize;
        let p: &Process = unsafe { &*(dummy as *const Process) };
        unsafe { size_of_val(p) }
    }

    pub fn create(&mut self, mapper: &mut paging::Map) -> Result<(), ProcessError> {
        mapper.clone_dir(&mut self.mapper);
        Ok(())
    }

    pub fn ppn(&self) -> u32 {
        self.mapper.ppn()
    }

    pub fn set_trap_frame(&mut self, tf: trap::TrapFrame) {
        self.trap_frame = tf;
    }

    pub fn region_alloc(
        &mut self,
        va: paging::VirtAddr,
        size: usize,
        flag: paging::Flag,
        allocator: &mut paging::Allocator,
    ) -> Result<(), ProcessError> {
        let old_satp = satp::SATP::read_csr();
        satp::SATP::set_ppn(self.ppn());
        for page in paging::Page::range(va, size as u32) {
            match allocator.alloc() {
                Ok(frame) => {
                    println!("{:?} -> {:?}", page, frame);
                    match self.mapper.map(page, frame, flag, allocator) {
                        Ok(_) => (),
                        Err(e) => return Err(ProcessError::FailedToMap(e)),
                    };
                }
                Err(e) => return Err(ProcessError::FailedToMap(e)),
            }
        }
        satp::SATP::set_ppn(old_satp);
        Ok(())
    }
    pub fn load_elf(
        &mut self,
        elf_file: &elf::Elf,
        allocator: &mut paging::Allocator,
    ) -> Result<(), ProcessError> {
        let old_satp = satp::SATP::read_csr();
        satp::SATP::set_ppn(self.ppn());

        for program in elf_file.programs() {
            /*println!(
                "{} -> {}: size {}",
                program.virt_addr, program.phys_addr, program.mem_size
            );*/
            self.region_alloc(
                program.virt_addr,
                utils::round_up(program.mem_size as u64, paging::PGSIZE as u64) as usize,
                /*program.flag,*/ // after region alloc, set flag
                paging::Flag::EXEC | paging::Flag::USER | paging::Flag::READ | paging::Flag::WRITE | paging::Flag::VALID,
                allocator,
            )?;
            let region = program.virt_addr.as_mut_ptr();
            unsafe {
                memutil::memset(
                    region,
                    0,
                    utils::round_up(program.mem_size as u64, paging::PGSIZE as u64) as usize,
                );
                memutil::memcpy(region, program.data, program.file_size);
            }
        }

        // alloc stack
        self.region_alloc(
            paging::VirtAddr::new(memlayout::USER_STACK_TOP),
            memlayout::USER_STACK_SIZE as usize,
            paging::Flag::VALID | paging::Flag::READ | paging::Flag::WRITE | paging::Flag::USER,
            allocator,
        )?;

        satp::SATP::set_ppn(old_satp);
        Ok(())
    }

    pub fn run(&mut self) -> ! {
        satp::SATP::set_ppn(self.ppn());
        self.status = Status::Running;
        unsafe {
            trap::pop_trap_frame(&self.trap_frame);
        }
    }
}

pub struct ProcessManager<'a> {
    procs: &'a mut [Process<'a>; N_PROCS],
    id_stack: [usize; N_PROCS],
    stack: usize,
}

impl<'a> ProcessManager<'a> {
    pub fn new(
        procs: &'a mut [Process<'a>; N_PROCS],
        proc_pages: &'a mut [paging::PageTable; N_PROCS],
        proc_tmp_pages: &'a mut [paging::PageTable; N_PROCS],
    ) -> ProcessManager<'a> {
        let mut id_stack = [0usize; N_PROCS];
        for (i, (p, t)) in proc_pages
            .iter_mut()
            .zip(proc_tmp_pages.iter_mut())
            .enumerate()
        {
            id_stack[i] = i;
            paging::PageTable::setup_tmp_table(p, t);
            procs[i].init(Id(i as u32), paging::Map::new(p, t));
            unsafe { procs[i].set_index(i) };
        }
        ProcessManager {
            procs,
            id_stack,
            stack: N_PROCS,
        }
    }
    pub unsafe fn alloc(&mut self) -> Result<*mut Process<'a>, ProcessError> {
        if self.stack == 0 {
            Err(ProcessError::FailedToCreateProcess)
        } else {
            self.stack -= 1;
            let id = self.id_stack[self.stack];
            Ok((&mut self.procs[id]) as *mut Process<'a>)
        }
    }

    pub fn dealloc(&mut self, proc: &Process) -> Result<(), ProcessError> {
        if self.stack == N_PROCS {
            Err(ProcessError::ProgramError("frame stack overflow"))
        } else {
            self.id_stack[self.stack] = proc.index;
            self.stack += 1;
            Ok(())
        }
    }
}
