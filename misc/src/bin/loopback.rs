#![no_std]
#![no_main]
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
