#![no_main]
#![no_std]

#[macro_use]
extern crate misc;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("{}", "Hello Syscall");
    misc::syscall::sys_exit(0);
    loop {}
}