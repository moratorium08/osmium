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

use core::panic::PanicInfo;

extern "C" {
    static kernel_end: u8;
}

fn get_kernel_end_addr() -> u64 {
    unsafe { (&kernel_end as *const u8) as u64 }
}

#[no_mangle]
pub extern "C" fn __start_rust() -> ! {
    let kern_pgdir = &mut paging::PageTable::gen_recursive();
    let mut mapper = paging::Map::new(kern_pgdir);
    println!("kernel ends with {:x}", get_kernel_end_addr());
    if let Err(e) = mapper.map_region(
        paging::VirtAddr::new(0),
        paging::PhysAddr::new(0),
        get_kernel_end_addr() as usize,
        paging::Flag::VALID | paging::Flag::READ | paging::Flag::WRITE,
    ) {
        panic!("Failed to map kernel region");
    }

    csr::SATP::enable_paging();

    println!("ok");
    loop {}
}

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
