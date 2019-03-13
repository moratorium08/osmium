use crate::elf;
use crate::files;
use crate::kernel;
use crate::memlayout;
use crate::paging;
use crate::proc;
use crate::trap;
use crate::uart;
use core::convert;
use core::slice;
use core::str;
use osmium_syscall::errors::SyscallError;
use osmium_syscall::number;
use osmium_syscall::perm;

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
    SendData {
        id: u32,
        data: u32,
    },
    ReceiveData {
        data_store: u32,
    },
    Map {
        src_id: u32,
        src_addr: u32,
        dst_id: u32,
        dst_addr: u32,
        perm: u32,
    },
    Alloc {
        addr: u32,
        size: u32,
        perm: u32,
    },
    Free {
        addr: u32,
        size: u32,
    },
}

impl convert::From<proc::ProcessError> for SyscallError {
    fn from(error: proc::ProcessError) -> Self {
        match error {
            proc::ProcessError::FailedToCreateProcess => SyscallError::NoMemorySpace,
            proc::ProcessError::NoSuchProcess => SyscallError::NotFound,
            proc::ProcessError::QueueIsEmpty => SyscallError::QueueIsEmpty,
            proc::ProcessError::QueueIsFull => SyscallError::QueueIsFull,
            proc::ProcessError::FailedToMap(_) | proc::ProcessError::ProgramError(_) => {
                SyscallError::InternalError
            }
        }
    }
}

impl convert::From<paging::PageError> for SyscallError {
    fn from(error: paging::PageError) -> Self {
        match error {
            paging::PageError::FailedToAllocMemory => SyscallError::NoMemorySpace,
            paging::PageError::PageIsNotMapped => SyscallError::NotFound,
            paging::PageError::IllegalAddress
            | paging::PageError::MapError
            | paging::PageError::AlreadyMapped
            | paging::PageError::ProgramError(_)
            | paging::PageError::NoMemory => SyscallError::InternalError,
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
            number::SYS_SEND => Ok(Syscall::SendData {
                id: tf.regs.a1(),
                data: tf.regs.a2(),
            }),
            number::SYS_RECEIVE => Ok(Syscall::ReceiveData {
                data_store: tf.regs.a1(),
            }),
            number::SYS_MMAP => Ok(Syscall::Map {
                src_id: tf.regs.a1(),
                src_addr: tf.regs.a2(),
                dst_id: tf.regs.a3(),
                dst_addr: tf.regs.a4(),
                perm: tf.regs.a5(),
            }),
            number::SYS_ALLOC => Ok(Syscall::Alloc {
                addr: tf.regs.a1(),
                size: tf.regs.a2(),
                perm: tf.regs.a3(),
            }),
            number::SYS_FREE => Ok(Syscall::Free {
                addr: tf.regs.a1(),
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

fn check_process_status(id: u32, k: &mut kernel::Kernel) -> Result<u32, SyscallError> {
    let p: &mut proc::Process = k.process_manager.id2proc(proc::Id(id))?;
    if p.parent_id != k.current_process.as_ref().unwrap().id {
        return Err(SyscallError::InvalidArguments);
    }
    Ok(p.status.to_u32())
}

fn send_data(id: u32, data: u32, k: &mut kernel::Kernel) -> Result<u32, SyscallError> {
    let p = k.process_manager.id2proc(proc::Id(id))?;
    let my_id = k.current_process.as_ref().unwrap().id;
    match p.enqueue_message(my_id, data) {
        Ok(()) => Ok(0),
        Err(proc::ProcessError::QueueIsFull) => Err(SyscallError::QueueIsFull),
        Err(_) => Err(SyscallError::InternalError),
    }
}

fn receive_data(ptr: u32, k: &mut kernel::Kernel) -> Result<u32, SyscallError> {
    let data_store: Option<&mut u32> = if ptr == 0 {
        None
    } else {
        // TODO: check access validity
        Some(unsafe { &mut *(ptr as *mut u32) })
    };
    match k.current_process.as_mut().unwrap().dequeue_message() {
        Ok(proc::Message { id, data }) => {
            match data_store {
                Some(store) => *store = data,
                None => (),
            };
            Ok(id.to_u32())
        }
        Err(proc::ProcessError::QueueIsEmpty) => Err(SyscallError::QueueIsEmpty),
        Err(_) => Err(SyscallError::InternalError),
    }
}

impl convert::From<perm::Perm> for paging::Flag {
    fn from(flag: perm::Perm) -> Self {
        let mut result = paging::Flag::empty();
        if flag.contains(perm::Perm::READ) {
            result |= paging::Flag::READ;
        }
        if flag.contains(perm::Perm::WRITE) {
            result |= paging::Flag::WRITE;
        }
        if flag.contains(perm::Perm::EXEC) {
            result |= paging::Flag::EXEC;
        }
        result
    }
}

fn mmap(
    src_id: u32,
    src_addr: u32,
    dst_id: u32,
    dst_addr: u32,
    perm_bits: u32,
    k: &mut kernel::Kernel,
) -> Result<u32, SyscallError> {
    println!(
        "mmap: {} {} {} {} {}",
        src_id, src_addr, dst_id, dst_addr, perm_bits
    );
    let src_p = k.process_manager.id2proc(proc::Id(src_id))?;
    let dst_p = k.process_manager.id2proc(proc::Id(dst_id))?;
    let src_addr = paging::VirtAddr::new(src_addr);
    let dst_addr = paging::VirtAddr::new(dst_addr);
    let p: perm::Perm = match perm::Perm::from_bits(perm_bits) {
        Some(x) => Ok(x),
        None => Err(SyscallError::InternalError),
    }?;

    let flag = paging::Flag::from(p);

    if !src_p.mapper.check_perm(src_addr, flag) {
        return Err(SyscallError::PermissionDenied);
    }

    if !src_addr.is_page_aligned() {
        return Err(SyscallError::InvalidAlignment);
    }

    if !dst_addr.is_page_aligned() {
        return Err(SyscallError::InvalidAlignment);
    }

    let frame: paging::Frame;

    address_space!(src_p, {
        let src_page = paging::Page::from_addr(src_addr);
        frame = src_p.mapper.frame(src_page)?;
    });

    address_space!(dst_p, {
        let dst_page = paging::Page::from_addr(dst_addr);
        dst_p.mapper.map(
            dst_page,
            frame,
            flag | paging::Flag::VALID | paging::Flag::USER,
            &mut k.allocator,
        )?;
    });
    Ok(0)
}

fn alloc(
    addr: u32,
    size: u32,
    perm_bits: u32,
    k: &mut kernel::Kernel,
) -> Result<u32, SyscallError> {
    let addr = if addr == 0 {
        k.current_process
            .as_ref()
            .unwrap()
            .mapper
            .search_free_addr(size)?
    } else {
        paging::VirtAddr::new(addr)
    };
    let size = size as u32;
    let p = match perm::Perm::from_bits(perm_bits) {
        Some(x) => Ok(x),
        None => Err(SyscallError::InternalError),
    }?;
    let flag = paging::Flag::from(p);
    k.current_process.as_mut().unwrap().mapper.alloc(
        addr,
        size,
        flag | paging::Flag::VALID | paging::Flag::USER,
        &mut k.allocator,
    )?;
    Ok(addr.to_u32())
}

fn free(addr: u32, size: u32, k: &mut kernel::Kernel) -> Result<u32, SyscallError> {
    unimplemented!()
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
        Syscall::SendData { id, data } => send_data(id, data, k),
        Syscall::ReceiveData { data_store } => receive_data(data_store, k),
        Syscall::Map {
            src_id,
            src_addr,
            dst_id,
            dst_addr,
            perm,
        } => mmap(src_id, src_addr, dst_id, dst_addr, perm, k),
        Syscall::Alloc { addr, size, perm } => alloc(addr, size, perm, k),
        Syscall::Free { addr, size } => free(addr, size, k),
    }
}
