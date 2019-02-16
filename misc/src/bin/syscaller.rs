#![no_std]
#![no_main]
#![feature(asm)]
extern crate misc;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    misc::puts("Please enter a byte");

    let mut buf = [0u8; 1];
    misc::sys_read(&mut buf, 1);
    misc::puts("Good job.");
    misc::sys_write(&buf, 1);

    misc::sys_write(&[misc::sys_get_proc_id() as u8], 1);
    misc::sys_yield();
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
