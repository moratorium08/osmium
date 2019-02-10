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

pub enum ProcessError {
    FailedToCreateProcess,
    ProgramError(&'static str),
}
#[derive(Copy, Clone)]
pub struct Id(pub u32);

#[repr(C)]
pub struct Process<'a> {
    pub table: Option<&'a mut paging::PageTable>,
    pub id: Id,
    pub parent_id: Id,
    index: usize,
    proc_type: Type,
    status: Status,
}

impl<'a> Process<'a> {
    pub fn init(&mut self, id: Id) {
        self.table = None;
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
            table: None,
            id: Id(0),
            parent_id: Id(0),
            index: 0,
            proc_type: Type::User,
            status: Status::Free,
        };
        unsafe { size_of_val(p) }
    }
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
    pub fn alloc(&mut self) -> Result<&'a mut Process, ProcessError> {
        if self.stack == 0 {
            Err(ProcessError::FailedToCreateProcess)
        } else {
            self.stack -= 1;
            let id = self.id_stack[self.stack];
            Ok(&mut self.procs[id])
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
