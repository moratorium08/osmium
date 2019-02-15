#![no_std]
#![no_main]
#![feature(asm)]
extern crate misc;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut buf = [0u8; 16];
    loop {
        let len = buf.len();
        misc::sys_read(&mut buf, len);
        misc::sys_write(&buf, len);
    }
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
