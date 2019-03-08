#![no_main]
#![feature(asm)]
#![no_std]

extern crate osmium_syscall;

#[macro_use]
pub mod uart;
pub mod syscall;

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
