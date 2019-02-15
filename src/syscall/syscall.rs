use super::number;
use crate::kernel;
use crate::trap;
use crate::uart;
use core::fmt;
use core::slice;

pub enum Syscall {
    UartWrite { buf: u32, size: u32 },
    UartRead { buf: u32, size: u32 },
    Exit { status: u32 },
}

#[derive(Debug, Copy, Clone)]
pub enum SyscallError {
    InvalidSyscallNumber,
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

/*pub fn mmap(va: u32, size: u32)*/

pub fn syscall_dispatch(sc: Syscall, k: &mut kernel::Kernel) -> Result<u32, SyscallError> {
    match sc {
        Syscall::UartRead { buf, size } => uart_read(buf, size),
        Syscall::UartWrite { buf, size } => uart_write(buf, size),
        Syscall::Exit { status } => exit(status, k),
    }
}
