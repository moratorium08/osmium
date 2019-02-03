#![feature(global_asm)]
#![no_std]
#![no_main]
#![feature(asm)]

#[macro_use]
extern crate bitflags;
extern crate array_init;

#[macro_use]
pub mod uart;
pub mod csr;
pub mod paging;
pub mod utils;

#[no_mangle]
pub extern "C" fn __start_rust() -> ! {
    println!("Hello println! val: {}", 1);
    unsafe {
        asm!("csrrs x4, satp, x0");
    }
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
