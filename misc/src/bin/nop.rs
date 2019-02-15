#![no_std]
#![no_main]
#![feature(asm)]

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        asm!("li a0, 10\n ecall\n");
    }
    loop {}
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
