use super::number;
use crate::kernel;
use crate::proc;
use crate::trap;
use crate::uart;
use core::fmt;
use core::slice;

pub enum Syscall {
    UartWrite { buf: u32, size: u32 },
    UartRead { buf: u32, size: u32 },
    Exit { status: u32 },
    GetProcId,
    Yield,
    Fork,
}

#[derive(Debug, Copy, Clone)]
pub enum SyscallError {
    InvalidSyscallNumber,
    InternalError,
    TooManyProcess,
    NoMemorySpace,
}

impl fmt::Display for SyscallError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl SyscallError {
    pub fn to_str(&self) -> &'static str {
        match self {
            InvalidSyscallNumber => "Invalid Syscall Number",
        }
    }
}

impl Syscall {
    pub fn from_trap_frame(tf: &trap::TrapFrame) -> Result<Syscall, SyscallError> {
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
    k.current_process
        .as_mut()
        .unwrap()
        .mapper
        .create_cow_user_memory(&mut process.mapper, &mut k.allocator);

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

pub fn syscall_dispatch(
    sc: Syscall,
    k: &mut kernel::Kernel,
    tf: &trap::TrapFrame,
) -> Result<u32, SyscallError> {
    match sc {
        Syscall::UartRead { buf, size } => uart_read(buf, size),
        Syscall::UartWrite { buf, size } => uart_write(buf, size),
        Syscall::Exit { status } => exit(status, k),
        Syscall::GetProcId => get_proc_id(k),
        Syscall::Yield => yield_process(k),
        Syscall::Fork => fork(k, tf),
    }
}
