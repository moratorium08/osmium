#![no_std]
#![no_main]
#![feature(asm)]
extern crate misc;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    misc::sys_exit(0)
}

use core::panic::PanicInfo;
#[panic_handler]
#[no_mangle]
pub fn panic(info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn abort() -> ! {
    loop {}
}
