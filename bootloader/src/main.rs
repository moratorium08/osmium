#![feature(global_asm)]
#![feature(core_intrinsics)]
#![no_std]
#![no_main]
#![feature(asm)]

use core::panic::PanicInfo;
use core::fmt::Write;

#[no_mangle]
pub static KERN_START: usize = 0x50000;

const UART_RX: *const u8 = 0x80000000 as *const u8;
const UART_TX: *mut u8 = 0x80000004 as *mut u8;

#[inline(never)]
pub fn read_byte() -> u8 {
    unsafe { *UART_RX }
}

struct UART;

fn write_byte(byte: u8) {
    unsafe {
        *UART_TX = byte;
    }
}

impl Write for UART {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            write_byte(c);
        }
        Ok(())
    }
}

pub fn print(arg: ::core::fmt::Arguments) {
    UART.write_fmt(arg).expect("failed to send by UART");
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($arg:expr) => (print!(concat!($arg, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
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

global_asm!(
    r#"
.global boot_time_trap_handler 
boot_time_trap_handler:
    csrrw x0, sscratch, sp
    lui     sp, %hi(0x7000000)
    addi    sp, sp, %lo(0x7000000)
    addi sp, sp, -128
    sw x0, 0(sp)
    sw x1, 4(sp)
    
    csrrs x1, scause, x0 
    sw x1, 8(sp)

    sw x3, 12(sp)
    sw x4, 16(sp)
    sw x5, 20(sp)
    sw x6, 24(sp)
    sw x7, 28(sp)
    sw x8, 32(sp)
    sw x9, 36(sp)
    sw x10, 40(sp)
    sw x11, 44(sp)
    sw x12, 48(sp)
    sw x13, 52(sp)
    sw x14, 56(sp)
    sw x15, 60(sp)
    sw x16, 64(sp)
    sw x17, 68(sp)
    sw x18, 72(sp)
    sw x19, 76(sp)
    sw x20, 80(sp)
    sw x21, 84(sp)
    sw x22, 88(sp)
    sw x23, 92(sp)
    sw x24, 96(sp)
    sw x25, 100(sp)
    sw x26, 104(sp)
    sw x27, 108(sp)
    sw x28, 112(sp)
    sw x29, 116(sp)
    sw x30, 120(sp)
    sw x31, 124(sp)

    mv a0, sp
    call rust_boot_handler 
"#
);

#[no_mangle]
extern "C" fn rust_boot_handler() -> ! {
    let sepc: u32;
    let scause: u32;
    let stval: u32;
    let sstatus: u32;
    let sie: u32;
    let sp: u32;
    unsafe {
        asm!(
            "
        csrrs $0, sepc, x0\n
        csrrs $1, scause, x0\n
        csrrs $2, stval, x0\n
        csrrs $3, sstatus, x0\n
        csrrs $4, sie, x0\n
        mv $5, sp\n
    "
        : "=&r"(sepc), "=&r"(scause), "=&r"(stval), "=&r"(sstatus), "=&r"(sie), "=&r"(sp)
            );
    }
    println!(
        "[store]sepc = {:x}, scause = {:x}, stval = {:x}\nsstatus = {:x}, sie = {:x}, sp = {:x}",
        sepc, scause, stval, sstatus, sie, sp
    );
    if scause == (1 << 17) {
        unsafe {
            asm!(
                "
                lw x0, 0(sp)\n
                lw x1, 4(sp)\n
                lw x3, 12(sp)\n
                lw x4, 16(sp)\n
                lw x5, 20(sp)\n
                lw x6, 24(sp)\n
                lw x7, 28(sp)\n
                lw x8, 32(sp)\n
                lw x9, 36(sp)\n
                lw x10, 40(sp)\n
                lw x11, 44(sp)\n
                lw x12, 48(sp)\n
                lw x13, 52(sp)\n
                lw x14, 56(sp)\n
                lw x15, 60(sp)\n
                lw x16, 64(sp)\n
                lw x17, 68(sp)\n
                lw x18, 72(sp)\n
                lw x19, 76(sp)\n
                lw x20, 80(sp)\n
                lw x21, 84(sp)\n
                lw x22, 88(sp)\n
                lw x23, 92(sp)\n
                lw x24, 96(sp)\n
                lw x25, 100(sp)\n
                lw x26, 104(sp)\n
                lw x27, 108(sp)\n
                lw x28, 112(sp)\n
                lw x29, 116(sp)\n
                lw x30, 120(sp)\n
                lw x31, 124(sp)\n

                addi sp, sp, 128\n

                csrrs sp, sscratch, x0\n
                sret
            "
            );
        }
    }

    panic!("boot error. bye")
}

fn setup_boot_time_trap() {
    unsafe {
        asm!(
            "
        lui     a0, %hi(boot_time_trap_handler)
        addi    a0, a0, %lo(boot_time_trap_handler)
        slli    a0, a0, 2\n
        csrrs   x0, stvec, a0\n
        "
        );
    }
}

#[no_mangle]
pub extern "C" fn __start_rust() -> ! {
    println!("setup");
    //setup_boot_time_trap();
    let size = read_u32() as usize;
    let mut addr = KERN_START;
    let mut target = (KERN_START + 4) as *const u32;

    let s = size / 4 / 100;
    for i in 0..size / 4 {
        let ptr = addr as *mut u32;
        let data = read_u32();
        if i % s == 0 {
            println!("{}%", i * 100 * 4 / size);
        }
        unsafe {
            *ptr = u32::from_be(data);
        }
        addr += 4;
    }
    unsafe {
        asm!(
            "
            lui a0, %hi(0x80000004)
            addi a0, a0, %lo(0x80000004)

            addi a1, x0, 111
            sw a1, 0(a0)
            addi a1, x0, 107
            sw a1, 0(a0)
            addi a1, x0, 10
            sw a1, 0(a0)
        "
        );

        let sp: u32;
        unsafe {
            asm!("
                mv $0, sp"
                :"=&r"(sp));
        }

        println!("{:x} {:x} {:x}", addr, unsafe { *target }, sp);

        asm!(
            "
            lui a0, %hi(KERN_START)
            addi a0, a0, %lo(KERN_START)
            lw a0, 0(a0)
            jalr x0, a0, 0
        "
        );
    }
    loop {}
}

#[cfg(not(test))]
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
