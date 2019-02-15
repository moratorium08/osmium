#![no_std]
#![no_main]
#![feature(asm)]

fn sys_write(buf: &[u8], size: usize) {
    unsafe {
        asm!("
        li a0, 0\n 
        mv a1, $0\n
        mv a2, $1\n
        ecall\n
        "
        :
        : "r"(buf.as_ptr() as u32), "r"(size as u32));
    }

}
fn sys_dummy(){
    unsafe {
        asm!("
        li a0, 100\n 
        ecall\n
        ");
    }
}


#[no_mangle]
pub extern "C" fn _start() -> ! {
    let buf = "Hello Syscall";

    sys_write(buf.as_bytes(), buf.len());
    sys_dummy();

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
