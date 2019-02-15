use super::number;
use crate::trap;
use crate::uart;
use core::fmt;
use core::slice;

pub enum Syscall {
    UartWrite { buf: u32, size: u32 },
    UartRead { buf: u32, size: u32 },
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
    println!("{:x}, {}", buf, size);
    // TODO: check buf's validity
    let buf: &mut [u8] = unsafe { slice::from_raw_parts_mut(buf as *mut u8, size as usize) };
    for c in buf {
        print!("{}", *c as char);
    }
    Ok(size)
}
