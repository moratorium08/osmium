#![no_main]
#![no_std]
#![feature(asm)]

#[macro_use]
extern crate misc;
extern crate osmium_syscall;

use misc::syscall;
use osmium_syscall::errors::SyscallError;

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
    Ok(())
}

fn child() -> Result<(), SyscallError> {
    let id = syscall::sys_receive()?.data;
    // should check id is my parent
    let mut data = [1; SIZE];
    loop {
        for i in 0..SIZE {
            let v = receive_from_id(id)?;
            data[i] = v + 1;
        }
        for x in &data {
            blocking_send(id, *x)?;
        }
    }
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
