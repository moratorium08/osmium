#![feature(global_asm)]
#![feature(core_intrinsics)]
#![no_std]
#![no_main]
#![feature(asm)]

use core::panic::PanicInfo;
#[no_mangle]
pub static KERN_START: usize = 0x4000;

use core::fmt::Write;

const UART_RX: *const u8 = 0x80000000 as *const u8;
const UART_TX: *mut u8 = 0x80000004 as *mut u8;

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

pub fn read_byte() -> u8 {
    unsafe { *UART_RX }
}

pub fn read_u32() -> u32 {
    let mut data = 0u32; 
    for _ in 0..4 {
        let b = read_byte();
        data = (data << 8) | (b as u32);
    }
    data
}

#[no_mangle]
pub extern "C" fn __start_rust() -> ! {
    let size = read_u32();
    let mut addr = KERN_START;
    for _ in 0..size / 4 {
        let ptr = addr as *mut u32;
        unsafe {
            *ptr = read_u32();
        }
        addr += 4;
    }
    unsafe {
        asm!("
            lui a0, %hi(KERN_START)
            addi a0, a0, %lo(KERN_START)
            jalr x0, a0, 0
        ");
    }
    loop{}
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

