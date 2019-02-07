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
    static mut kernel_pgdir_ptr: u32;
    static mut kernel_frames_ptr: u32;
}

const IO_REGION: u64 = 0x80000000;

fn get_kernel_end_addr() -> u64 {
    unsafe { (&kernel_end as *const u8) as u64 }
}

#[no_mangle]
pub extern "C" fn __start_rust() -> ! {
    println!("hello\n\n");
    let kern_pgdir = unsafe { paging::PageTable::gen_recursive(&mut kernel_pgdir_ptr as *mut u32) };
    let kern_pgdir_addr = (kern_pgdir as *const paging::PageTable) as u32;

    let kernel_frames = unsafe { (&mut kernel_frames_ptr as *mut u32) };
    println!("kern frames addr {:p}", kernel_frames);

    let mut mapper = paging::Map::new(kern_pgdir);
    println!("mapper created");

    let mut allocator = unsafe { paging::Allocator::new(kernel_frames) };
    println!("allocator created");

    println!("kernel ends with {:x}", get_kernel_end_addr());
    if let Err(e) = mapper.boot_map_region(
        paging::VirtAddr::new(0),
        paging::PhysAddr::new(0),
        get_kernel_end_addr() as usize,
        paging::Flag::VALID | paging::Flag::READ | paging::Flag::WRITE,
        &mut allocator,
    ) {
        panic!("Failed to map kernel region. Reason: {:?}", e);
    }
    println!("mapping created");

    if let Err(e) = mapper.boot_map_region(
        paging::VirtAddr::new(IO_REGION as u32),
        paging::PhysAddr::new(IO_REGION),
        paging::PGSIZE,
        paging::Flag::READ | paging::Flag::WRITE | paging::Flag::VALID,
        &mut allocator,
    ) {
        panic!("Failed to map io region. Reason: {:?}", e);
    }
    println!("mapping created");

    csr::SATP::set_ppn(kern_pgdir_addr >> paging::LOG_PGSIZE);
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
