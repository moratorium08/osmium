#![no_main]
#![no_std]
#![feature(asm)]

#[macro_use]
extern crate misc;

use misc::syscall;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut buf = [0u8; 16];
    loop {
        let len = buf.len();
        syscall::sys_read(&mut buf, len);
        syscall::sys_write(&buf, len);
    }
}