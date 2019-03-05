#![no_main]
#![feature(asm)]
#![no_std]

extern crate osmium_syscall;

#[macro_use]
pub mod uart;
pub mod syscall;
