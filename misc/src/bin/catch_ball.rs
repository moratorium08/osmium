#![no_main]
#![no_std]
#![feature(asm)]

#[macro_use]
extern crate misc;
extern crate osmium_syscall;

use core::slice;
use misc::syscall;
use osmium_syscall::errors::SyscallError;
use osmium_syscall::perm;

fn blocking_receive() -> Result<syscall::Message, SyscallError> {
    //println!("blocking_receive");
    match syscall::sys_receive() {
        Err(SyscallError::QueueIsEmpty) => {
            syscall::sys_yield();
            blocking_receive()
        }
        Err(e) => Err(e),
        Ok(msg) => Ok(msg),
    }
}

fn receive_from_id(id: u32) -> Result<u32, SyscallError> {
    match blocking_receive() {
        Ok(ref msg) if msg.id == id => Ok(msg.data),
        Ok(ref msg) => receive_from_id(id),
        Err(e) => Err(e),
    }
}

fn blocking_send(id: u32, data: u32) -> Result<(), SyscallError> {
    match syscall::sys_send(id, data) {
        Err(SyscallError::QueueIsFull) => {
            syscall::sys_yield();
            blocking_send(id, data)
        }
        Err(e) => Err(e),
        Ok(()) => Ok(()),
    }
}

const SIZE: usize = 8;
fn parent(child_id: u32) -> Result<(), SyscallError> {
    let my_id = syscall::sys_get_proc_id();
    syscall::sys_send(child_id, my_id)?;

    let mut data = [1; SIZE];
    for i in 0..5 {
        for x in &data {
            blocking_send(child_id, *x)?;
        }
        for i in 0..SIZE {
            let v = receive_from_id(child_id)?;
            data[i] = (i as u32 + 1) * v;
        }
    }
    for i in 0..SIZE {
        println!("{}: {}", i, data[i]);
    }
    println!("let's start sharing page");

    // first, create memory region.
    let addr = syscall::sys_alloc(None, 4096 * 2, perm::Perm::READ | perm::Perm::WRITE)?;
    println!("addr: {}", addr);

    // second, memory map between the parent and its child
    syscall::sys_mmap(
        my_id,
        addr,
        child_id,
        addr,
        perm::Perm::READ | perm::Perm::WRITE,
    )?;

    // then, notify the address where they use
    syscall::sys_send(child_id, addr)?;

    // start communicating.
    let data = unsafe { slice::from_raw_parts_mut(addr as *mut u32, 4096 * 2 / 4) };
    data[0] = 100;
    //data[1024] = 100;
    // notify the timing
    syscall::sys_send(child_id, addr)?;
    let _ = receive_from_id(child_id)?;
    Ok(())
}

fn child() -> Result<(), SyscallError> {
    let id = syscall::sys_receive()?.data;
    // should check id is my parent
    let mut data = [1; SIZE];
    for _ in 0..5 {
        for i in 0..SIZE {
            let v = receive_from_id(id)?;
            data[i] = v + 1;
        }
        for x in &data {
            blocking_send(id, *x)?;
        }
    }

    let addr = receive_from_id(id)?;
    println!("child addr: {}", addr);
    let data = unsafe { slice::from_raw_parts_mut(addr as *mut u32, 4096 / 4) };

    // wait for update
    let _ = receive_from_id(id)?;
    println!("data[0] = {}", data[0]);

    syscall::sys_send(id, addr)?;

    Ok(())
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    match syscall::sys_fork() {
        syscall::ForkResult::Parent(id) => match parent(id) {
            Ok(()) => (),
            Err(e) => println!("parent error: {}", e),
        },
        syscall::ForkResult::Fail => {
            syscall::sys_exit(-1);
        }
        syscall::ForkResult::Child => match child() {
            Err(SyscallError::NotFound) | Ok(()) => (),
            Err(e) => println!("child error: {}", e),
        },
    }
    syscall::sys_exit(0)
}
