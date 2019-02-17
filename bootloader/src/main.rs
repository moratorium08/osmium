#![feature(global_asm)]
#![feature(core_intrinsics)]
#![no_std]
#![no_main]
#![feature(asm)]

use core::panic::PanicInfo;
#[no_mangle]
pub static KERN_START: usize = 0x10000;

const UART_RX: *const u8 = 0x80000000 as *const u8;

#[inline(never)]
pub fn read_byte() -> u8 {
    unsafe { *UART_RX }
}

pub fn read_u32() -> u32 {
    let mut data = 0u32; 
    let b = read_byte();
    data = (data << 8) | (b as u32);
    let b = read_byte();
    data = (data << 8) | (b as u32);
    let b = read_byte();
    data = (data << 8) | (b as u32);
    let b = read_byte();
    data = (data << 8) | (b as u32);
    data
}

#[no_mangle]
pub extern "C" fn __start_rust() -> ! {
    let size = read_u32() as usize;
    let mut addr = KERN_START;
    for i in 0..size / 4 {
        if (i % (KERN_START / 100) == 0) {
            unsafe {
            asm!("
                lui a0, %hi(0x80000004)
                addi a0, a0, %lo(0x80000004)

                addi a1, x0, 111
                sw a1, 0(a0)
                ");
            }
        }
        let ptr = addr as *mut u32;
        unsafe {
            *ptr = u32::from_be(read_u32());
        }
        addr += 4;
    }
    unsafe {
        asm!("
            lui a0, %hi(0x80000004)
            addi a0, a0, %lo(0x80000004)

            addi a1, x0, 111
            sw a1, 0(a0)
            addi a1, x0, 107
            sw a1, 0(a0)
            addi a1, x0, 10
            sw a1, 0(a0)

            lui a0, %hi(KERN_START)
            addi a0, a0, %lo(KERN_START)
            lw a0, 0(a0)
            jalr x0, a0, 0
        ");
    }
    loop{}
}

#[panic_handler]
#[no_mangle]
pub fn panic(info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn abort() -> ! {
    loop {}
}

