#![no_main]
#![no_std]
#![feature(asm)]

#[macro_use]
extern crate misc;

use core::str;
use misc::syscall;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Please enter a byte");

    let mut buf = [0u8; 1];
    syscall::sys_read(&mut buf, 1);
    println!("Good job > {}", str::from_utf8(&buf).unwrap());

    println!("My proc id is {:x}", syscall::sys_get_proc_id());

    match syscall::sys_fork() {
        syscall::ForkResult::Parent(id) => {
            println!("I am a parent of {:x}", id);
        },
        syscall::ForkResult::Fail => {
            println!("fork failed");
        },
        syscall::ForkResult::Child => {
            println!("I'm a child!! ogya-");
        }
    }

    println!("Yield!");
    syscall::sys_yield();
    println!("Goodbye");
    syscall::sys_exit(0)
}

use core::panic::PanicInfo;
#[panic_handler]
#[no_mangle]
pub fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub extern "C" fn abort() -> ! {
    loop {}
}
