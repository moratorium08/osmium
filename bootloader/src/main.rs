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

#[no_mangle]
pub extern "C" fn boot_time_trap_handler() -> ! {
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
        "sepc = {:x}, scause = {:x}, stval = {:x}\nsstatus = {:x}, sie = {:x}, sp = {:x}",
        sepc,
        scause,
        stval,
        sstatus,
        sie,
        sp
    );
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
    setup_boot_time_trap();
    let size = read_u32() as usize;
    let mut addr = KERN_START;
    let mut target = (KERN_START + 4) as *const u32;
    for i in 0..size / 4 {
        let ptr = addr as *mut u32;
        let data = read_u32();
        if (i >= 2460 && i < 0xa3d && i % 3 == 0) {
            let sp: u32;
            unsafe {
                asm!("
                    mv $0, sp"
                    :"=&r"(sp));
            }
            println!(
                "{:x} {:x} {:x} {:x} {:x}",
                i,
                addr,
                data,
                unsafe { *target },
                sp
            );
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
