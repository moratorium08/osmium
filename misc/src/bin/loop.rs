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
