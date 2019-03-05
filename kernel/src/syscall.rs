use crate::elf;
use crate::files;
use crate::kernel;
use crate::memlayout;
use crate::memutil;
use crate::proc;
use crate::trap;
use crate::uart;
use core::fmt;
use core::slice;
use core::str;
use osmium_syscall::number;

#[derive(Copy, Clone, Debug)]
pub enum Syscall {
    UartWrite {
        buf: u32,
        size: u32,
    },
    UartRead {
        buf: u32,
        size: u32,
    },
    Exit {
        status: u32,
    },
    GetProcId,
    Yield,
    Fork,
    Execve {
        filename: u32,
        filename_length: u32,
        argv: u32,
        envp: u32,
    },
    CheckProcessStatus {
        id: u32,
    },
}

#[derive(Debug, Copy, Clone)]
pub enum SyscallError {
    InvalidSyscallNumber,
    InternalError,
    TooManyProcess,
    NoMemorySpace,
    InvalidArguments,
    IllegalFile,
    NotFound,
}

impl SyscallError {
    pub fn to_syscall_result(&self) -> i32 {
        match self {
            SyscallError::InvalidSyscallNumber => -1,
            SyscallError::InternalError => -2,
            SyscallError::TooManyProcess => -3,
            SyscallError::NoMemorySpace => -4,
            SyscallError::InvalidArguments => -5,
            SyscallError::IllegalFile => -6,
            SyscallError::NotFound => -7,
        }
    }
}

impl fmt::Display for SyscallError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl SyscallError {
    pub fn to_str(&self) -> &'static str {
        match self {
            SyscallError::InvalidSyscallNumber => "Invalid Syscall Number",
            SyscallError::InternalError => "Internal error",
            SyscallError::TooManyProcess => "Too many process",
            SyscallError::NoMemorySpace => "No Memory Space",
            SyscallError::InvalidArguments => "Invalid Arguments",
            SyscallError::IllegalFile => "Illegal File",
            SyscallError::NotFound => "Not Found",
        }
    }
}

impl Syscall {
    pub fn from_trap_frame(tf: &trap::TrapFrame) -> Result<Syscall, SyscallError> {
        dprintln!("syscall number: {:x}", tf.regs.a0());
        match tf.regs.a0() {
            number::SYS_UART_READ => Ok(Syscall::UartRead {
                buf: tf.regs.a1(),
                size: tf.regs.a2(),
            }),
            number::SYS_UART_WRITE => Ok(Syscall::UartWrite {
                buf: tf.regs.a1(),
                size: tf.regs.a2(),
            }),
            number::SYS_EXIT => Ok(Syscall::Exit {
                status: tf.regs.a1(),
            }),
            number::SYS_GET_PROC_ID => Ok(Syscall::GetProcId),
            number::SYS_YIELD => Ok(Syscall::Yield),
            number::SYS_FORK => Ok(Syscall::Fork),
            number::SYS_EXECVE => Ok(Syscall::Execve {
                filename: tf.regs.a1(),
                filename_length: tf.regs.a2(),
                argv: tf.regs.a3(),
                envp: tf.regs.a4(),
            }),
            number::SYS_PROC_STATUS => Ok(Syscall::CheckProcessStatus { id: tf.regs.a1() }),
            _ => Err(SyscallError::InvalidSyscallNumber),
        }
    }
}

pub fn uart_read(buf: u32, size: u32) -> Result<u32, SyscallError> {
    // TODO: check buf's validity
    let buf: &mut [u8] = unsafe { slice::from_raw_parts_mut(buf as *mut u8, size as usize) };
    for c in buf.iter_mut() {
        *c = uart::read_byte();
    }
    Ok(size)
}

pub fn uart_write(buf: u32, size: u32) -> Result<u32, SyscallError> {
    // TODO: check buf's validity
    let buf: &mut [u8] = unsafe { slice::from_raw_parts_mut(buf as *mut u8, size as usize) };
    for c in buf {
        print!("{}", *c as char);
    }
    Ok(size)
}

pub fn exit(status: u32, kernel: &mut kernel::Kernel) -> Result<u32, SyscallError> {
    match kernel.current_process {
        Some(ref mut p) => p.exit(status),
        None => (),
    };
    kernel.current_process = None;
    Ok(0)
}

pub fn get_proc_id(k: &kernel::Kernel) -> Result<u32, SyscallError> {
    match k.current_process {
        Some(ref p) => Ok(p.id.to_u32()),
        None => Err(SyscallError::InternalError),
    }
}

pub fn yield_process(k: &mut kernel::Kernel) -> Result<u32, SyscallError> {
    k.current_process.as_mut().unwrap().status = proc::Status::Runnable;
    k.current_process = None;
    Ok(0)
}

pub fn fork(k: &mut kernel::Kernel, tf: &trap::TrapFrame) -> Result<u32, SyscallError> {
    // create new process
    let process: &mut proc::Process;

    match unsafe { k.process_manager.alloc() } {
        Ok(p) => {
            process = unsafe { &mut *p };
        }
        Err(e) => {
            return Err(match e {
                proc::ProcessError::FailedToCreateProcess => SyscallError::TooManyProcess,
                _ => SyscallError::InternalError,
            });
        }
    };
    match process.create(&mut k.mapper) {
        Ok(()) => (),
        Err(e) => return Err(SyscallError::NoMemorySpace),
    };
    // setup CoW and dup page table
    match k
        .current_process
        .as_mut()
        .unwrap()
        .mapper
        .create_cow_user_memory(&mut process.mapper, &mut k.allocator)
    {
        Ok(()) => (),
        Err(e) => {
            dprintln!("failed to create cow: {}", e);
            return Err(SyscallError::InternalError);
        }
    }

    // change status
    process.status = proc::Status::Runnable;
    // set child's tf. and
    let mut new_tf = tf.clone();
    new_tf.regs.set_syscall_result(0);
    process.trap_frame = new_tf;
    process.parent_id = k.current_process.as_ref().unwrap().id;
    // parent's retval is child's proc id
    Ok(process.id.to_u32())
}

fn execve(
    filename: u32,
    filename_length: u32,
    argv: u32,
    envp: u32,
    tf: &mut trap::TrapFrame,
    k: &mut kernel::Kernel,
) -> Result<u32, SyscallError> {
    let name: &str;
    unsafe {
        // TODO: user memcheck
        match str::from_utf8(slice::from_raw_parts(
            filename as *const u8,
            filename_length as usize,
        )) {
            Ok(f) => name = f,
            Err(_) => {
                return Err(SyscallError::InvalidArguments);
            }
        };
    }

    let file = match files::search(name) {
        Some(file) => file,
        None => return Err(SyscallError::NotFound),
    };

    let e = match elf::Elf::new(file.bytes) {
        Ok(e) => e,
        Err(_) => return Err(SyscallError::IllegalFile),
    };
    match k
        .current_process
        .as_mut()
        .unwrap()
        .load_elf(&e, &mut k.allocator)
    {
        Ok(()) => (),
        Err(e) => return Err(SyscallError::IllegalFile),
    };
    dprintln!("set entry point: {:x}", e.elf.entry);
    let new_tf = trap::TrapFrame::new(e.elf.entry, memlayout::USER_STACK_BOTTOMN);
    *tf = new_tf;
    Ok(0)
}

pub fn check_process_status(id: u32, k: &mut kernel::Kernel) -> Result<u32, SyscallError> {
    let p: &mut proc::Process;
    match unsafe { k.process_manager.id2proc(proc::Id(id)) } {
        Ok(ptr) => p = unsafe { &mut *ptr },
        Err(_) => return Err(SyscallError::InvalidArguments),
    }
    if p.parent_id != k.current_process.as_ref().unwrap().id {
        return Err(SyscallError::InvalidArguments);
    }
    Ok(p.status.to_u32())
}

pub fn syscall_dispatch(
    sc: Syscall,
    k: &mut kernel::Kernel,
    tf: &mut trap::TrapFrame,
) -> Result<u32, SyscallError> {
    dprintln!("{:?}", sc);
    match sc {
        Syscall::UartRead { buf, size } => uart_read(buf, size),
        Syscall::UartWrite { buf, size } => uart_write(buf, size),
        Syscall::Exit { status } => exit(status, k),
        Syscall::GetProcId => get_proc_id(k),
        Syscall::Yield => yield_process(k),
        Syscall::Fork => fork(k, tf),
        Syscall::Execve {
            filename,
            filename_length,
            argv,
            envp,
        } => execve(filename, filename_length, argv, envp, tf, k),
        Syscall::CheckProcessStatus { id } => check_process_status(id, k),
    }
}
