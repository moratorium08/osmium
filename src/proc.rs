use core::fmt;
use paging;

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
    ProgramError(&'static str),
}

impl ProcessError {
    fn to_str(&self) -> &'static str {
        match self {
            ProcessError::FailedToCreateProcess => "failed to create process",
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
    pub mapper: Option<paging::Map<'a>>,
    pub id: Id,
    pub parent_id: Id,
    index: usize,
    proc_type: Type,
    status: Status,
}

impl<'a> Process<'a> {
    pub fn init(&mut self, id: Id) {
        self.mapper = None;
        self.id = id;
        self.parent_id = id;
        self.proc_type = Type::User;
        self.status = Status::Free;
    }
    // dont touch without ProcessManager
    pub unsafe fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    pub fn size_of() -> usize {
        use core::intrinsics::size_of_val;
        let p = &Process {
            mapper: None,
            id: Id(0),
            parent_id: Id(0),
            index: 0,
            proc_type: Type::User,
            status: Status::Free,
        };
        unsafe { size_of_val(p) }
    }

    pub fn create(
        &mut self,
        allocator: &mut paging::Allocator,
        mapper: &mut paging::Map,
    ) -> Result<(), ProcessError> {
        let frame = if let Ok(frame) = allocator.alloc() {
            frame
        } else {
            return Err(ProcessError::FailedToCreateProcess);
        };
        println!("are");
        let a = frame.phys_addr().to_u64();
        let table: &mut paging::PageTable =
            unsafe { &mut (*frame.phys_addr().as_mut_kernel_ptr()) };
        // register kernel address space
        mapper.identity_map(
            frame,
            paging::Flag::READ | paging::Flag::WRITE | paging::Flag::VALID,
            allocator,
        );
        println!("neko");
        mapper.clone_dir(table);
        println!("hoge");
        table.set_recursive_entry(frame);
        self.mapper = Some(paging::Map::new(table));
        Ok(())
    }

    /*fn copy_kernel_table(&mut self, kernel_table: &paging::PageTable) {
            let table = match self.mapper {
                Some(ref mut m) => m,
                None => panic!("copy kernel table failed"),
            };
    }*/
}

pub struct ProcessManager<'a> {
    procs: &'a mut [Process<'a>; N_PROCS],
    id_stack: [usize; N_PROCS],
    stack: usize,
}

impl<'a> ProcessManager<'a> {
    pub fn new(procs: &'a mut [Process<'a>; N_PROCS]) -> ProcessManager<'a> {
        let mut id_stack = [0usize; N_PROCS];
        for i in 0..N_PROCS {
            id_stack[i] = i;
            procs[i].init(Id(i as u32));
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
