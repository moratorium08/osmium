#![no_main]
#![no_std]
#![feature(asm)]

#[macro_use]
extern crate misc;

use misc::syscall;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}