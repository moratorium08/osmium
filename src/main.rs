#![feature(global_asm)]
#![feature(core_intrinsics)]
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
pub mod proc;
pub mod trap;
pub mod utils;

use core::panic::PanicInfo;
use csr::satp;

extern "C" {
    static kernel_end: u8;
    static mut kernel_pgdir_ptr: u32;
    static mut kernel_frames_ptr: u32;
}

const IO_REGION: u64 = 0x80000000;

fn get_kernel_end_addr() -> u64 {
    unsafe { (&kernel_end as *const u8) as u64 }
}

struct BootAlloc<'a> {
    pub procs: &'a mut [proc::Process<'a>; proc::N_PROCS],
}

// must call before memory management in order to reserve envs memory.
fn boot_alloc<'a>() -> (u64, BootAlloc<'a>) {
    let end = get_kernel_end_addr();

    let procs = unsafe { &mut *(end as *mut [proc::Process; proc::N_PROCS]) };
    (end + (proc::Process::size_of() as u64), BootAlloc { procs })
}

struct Kernel<'a> {
    kernel_pgdir: &'a mut paging::PageTable,
    mapper: paging::Map<'a>,
    allocator: paging::Allocator<'a>,
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

    let (kernel_memory_end, allocated) = boot_alloc();
    let is_used = |addr| {
        if (addr as u64) < kernel_memory_end + (paging::PGSIZE as u64) {
            return true;
        }
        if addr as u64 >= IO_REGION {
            return true;
        }
        false
    };
    let mut allocator = unsafe { paging::Allocator::new(kernel_frames, &is_used) };
    println!("allocator created");

    println!("envs start with {:x}", get_kernel_end_addr());
    if let Err(e) = mapper.boot_map_region(
        paging::VirtAddr::new(0),
        paging::PhysAddr::new(0),
        get_kernel_end_addr() as usize,
        paging::Flag::READ | paging::Flag::WRITE | paging::Flag::EXEC | paging::Flag::VALID,
        &mut allocator,
    ) {
        panic!("Failed to map kernel region. Reason: {:?}", e);
    }
    println!("kernel mapping created");

    if let Err(e) = mapper.boot_map_region(
        paging::VirtAddr::new(IO_REGION as u32),
        paging::PhysAddr::new(IO_REGION),
        paging::PGSIZE,
        paging::Flag::READ | paging::Flag::WRITE | paging::Flag::EXEC | paging::Flag::VALID,
        &mut allocator,
    ) {
        panic!("Failed to map io region. Reason: {:?}", e);
    }
    println!("io mapping created");

    satp::SATP::set_ppn(kern_pgdir_addr >> paging::LOG_PGSIZE);
    satp::SATP::enable_paging();

    println!("kernel space (identity) paging works!");

    println!("Let's create an user process");
    let mut process_manager = proc::ProcessManager::new(allocated.procs);
    let process = process_manager
        .alloc()
        .expect("failed to alloc process(program error)");
    //process.create(&mut allocator, &mut mapper);
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
