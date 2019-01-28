#![feature(global_asm)]
#![no_std]
#![no_main]

#[macro_use]
pub mod uart;


#[no_mangle]
pub extern "C" fn __start_rust() -> ! {
    println!("Hello println! val: {}", 1);
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
