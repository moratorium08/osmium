#![feature(global_asm)]
#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn __start_rust() -> ! {
    let uart = 0x10004 as *mut u8;
    for c in b"Hello from Rust!\n".iter() {
        unsafe {
            *uart = *c as u8;
        }
    }

    loop {}
}

use core::panic::PanicInfo;
#[panic_handler]
#[no_mangle]
pub fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn abort() -> ! {
    loop {}
}
