use core::fmt;
use osmium_syscall::errors::SyscallError;
use osmium_syscall::number;
use osmium_syscall::perm;

fn syscall_0(num: u32) -> u32 {
    let result: u32;
    unsafe {
        asm!("
            ecall
        "
        : "={x10}"(result)
        : "{x10}"(num)
        );
    }
    result
}

fn syscall_1(num: u32, a: u32) -> u32 {
    let result: u32;
    unsafe {
        asm!("
            ecall
        "
        : "={x10}"(result)
        : "{x10}"(num), "{x11}"(a)
        );
    }
    result
}

fn syscall_2(num: u32, a: u32, b: u32) -> u32 {
    let result: u32;
    unsafe {
        asm!("
            ecall
        "
        : "={x10}"(result)
        : "{x10}"(num), "{x11}"(a), "{x12}"(b)
        );
    }
    result
}

fn syscall_3(num: u32, a: u32, b: u32, c: u32) -> u32 {
    let result: u32;
    unsafe {
        asm!("
            ecall
        "
        : "={x10}"(result)
        : "{x10}"(num), "{x11}"(a), "{x12}"(b), "{x13}"(c)
        );
    }
    result
}

fn syscall_4(num: u32, a: u32, b: u32, c: u32, d: u32) -> u32 {
    let result: u32;
    unsafe {
        asm!("
            ecall
        "
        : "={x10}"(result)
        : "{x10}"(num), "{x11}"(a), "{x12}"(b), "{x13}"(c), "{x14}"(d)
        );
    }
    result
}

fn syscall_5(num: u32, a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 {
    let result: u32;
    unsafe {
        asm!("
            ecall
        "
        : "={x10}"(result)
        : "{x10}"(num), "{x11}"(a), "{x12}"(b), "{x13}"(c), "{x14}"(d), "{x15}"(e)
        );
    }
    result
}

pub fn sys_write(buf: &[u8], size: usize) -> u32 {
    syscall_2(number::SYS_UART_WRITE, buf.as_ptr() as u32, size as u32)
}

pub fn sys_read(buf: &mut [u8], size: usize) -> u32 {
    syscall_2(number::SYS_UART_READ, buf.as_ptr() as u32, size as u32)
}

pub fn sys_exit(status: i32) -> ! {
    syscall_1(number::SYS_EXIT, status as u32);
    loop {} // here does not reach
}

pub fn sys_get_proc_id() -> u32 {
    syscall_0(number::SYS_GET_PROC_ID)
}

pub fn sys_yield() -> u32 {
    syscall_0(number::SYS_YIELD)
}

pub enum ForkResult {
    Parent(u32),
    Child,
    Fail,
}
pub fn sys_fork() -> ForkResult {
    let r = syscall_0(number::SYS_FORK) as i32;
    if r < 0 {
        ForkResult::Fail
    } else if r == 0 {
        ForkResult::Child
    } else {
        ForkResult::Parent(r as u32)
    }
}

pub fn sys_execve(
    filename: &str,
    filename_len: u32,
    argv: &[*const u32],
    envp: &[*const u32],
) -> ! {
    let x = syscall_4(
        number::SYS_EXECVE,
        filename.as_bytes().as_ptr() as u32,
        filename_len,
        argv.as_ptr() as u32,
        envp.as_ptr() as u32,
    );
    if (x as i32) < 0 {
        println!("{}", SyscallError::from_syscall_result(x as i32));
        sys_exit(x as i32)
    }

    loop {}
}

pub enum ProcessStatus {
    Free,
    Running,
    Runnable,
    NotRunnable,
    Zonmbie,
}

impl ProcessStatus {
    pub fn from_u32(x: u32) -> ProcessStatus {
        match x {
            0 => ProcessStatus::Free,
            1 => ProcessStatus::Running,
            2 => ProcessStatus::Runnable,
            3 => ProcessStatus::NotRunnable,
            4 => ProcessStatus::Zonmbie,
            _ => panic!("failed to handle process status"),
        }
    }
}

pub fn sys_check_process_status(id: u32) -> ProcessStatus {
    let r = syscall_1(number::SYS_PROC_STATUS, id);
    ProcessStatus::from_u32(r)
}

pub fn sys_send(id: u32, data: u32) -> Result<(), SyscallError> {
    let r = syscall_2(number::SYS_SEND, id, data);
    let r = r as i32;
    if r < 0 {
        Err(SyscallError::from_syscall_result(r))
    } else {
        Ok(())
    }
}

pub struct Message {
    pub id: u32,
    pub data: u32,
}

pub fn sys_receive() -> Result<Message, SyscallError> {
    let mut data_store: u32 = 100;
    let r = syscall_1(number::SYS_RECEIVE, (&mut data_store) as *mut u32 as u32);
    let r = r as i32;
    if r < 0 {
        Err(SyscallError::from_syscall_result(r))
    } else {
        Ok(Message {
            id: r as u32,
            data: data_store,
        })
    }
}

pub fn sys_mmap(
    src_id: u32,
    src_addr: u32,
    dst_id: u32,
    dst_addr: u32,
    perm: perm::Perm,
) -> Result<(), SyscallError> {
    let r = syscall_5(
        number::SYS_MMAP,
        src_id,
        src_addr,
        dst_id,
        dst_addr,
        perm.bits(),
    ) as i32;
    if r < 0 {
        Err(SyscallError::from_syscall_result(r))
    } else {
        Ok(())
    }
}

pub fn sys_alloc(addr: Option<u32>, size: u32, perm: perm::Perm) -> Result<u32, SyscallError> {
    let addr_raw = match addr {
        Some(x) => x,
        None => 0,
    };
    let r = syscall_3(number::SYS_ALLOC, addr_raw, size, perm.bits()) as i32;
    if r < 0 && r > -4096 {
        Err(SyscallError::from_syscall_result(r))
    } else {
        Ok(r as u32)
    }
}

pub fn sys_free(addr: u32, size: u32) -> Result<(), SyscallError> {
    let r = syscall_2(number::SYS_FREE, addr, size) as i32;
    if r < 0 {
        Err(SyscallError::from_syscall_result(r))
    } else {
        Ok(())
    }
}
