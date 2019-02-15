use csr::{CSRRead, CSRWrite};

#[derive(Copy, Clone)]
pub enum Mode {
    Direct,
    Vectored,
    Reserved,
}

impl Mode {
    fn to_u32(self) -> u32 {
        match self {
            Mode::Direct => 0,
            Mode::Vectored => 1,
            Mode::Reserved => 2,
        }
    }
    fn from_u32(x: u32) -> Mode {
        match x {
            0 => Mode::Direct,
            1 => Mode::Vectored,
            _ => Mode::Reserved,
        }
    }
}

pub struct STVEC {
    pub mode: Mode,
    pub base: u32,
}
impl CSRRead for STVEC {
    fn read_csr() -> u32 {
        let result: u32;
        unsafe {
            asm!("csrrs $0, stvec, x0"
                : "=&r"(result));
        }
        result
    }
    fn from_u32(x: u32) -> STVEC {
        STVEC {
            mode: Mode::from_u32(x & 3),
            base: x >> 2,
        }
    }
}

impl CSRWrite for STVEC {
    fn read_and_write(val: u32) -> u32 {
        let result: u32;

        unsafe {
            asm!("csrrw $0, stvec, $1"
                : "=&r"(result)
                :   "r"(val));
        }
        result
    }

    fn write_csr(val: u32) {
        unsafe {
            asm!("csrrw x0, stvec, $0"
                :
                : "r"(val));
        }
    }

    fn bit_set(bitvec: u32) {
        unsafe {
            asm!("csrrs x0, stvec, $0"
                :
                : "r"(bitvec));
        }
    }

    fn bit_clear(bitvec: u32) {
        unsafe {
            asm!("csrrc x0, stvec, $0"
                :
                : "r"(bitvec));
        }
    }
    fn to_u32(&self) -> u32 {
        let mode = self.mode.to_u32();
        (self.base << 2) | mode
    }
}

impl STVEC {
    pub fn set_mode(mode: Mode) {
        let mut csr = STVEC::read();
        csr.mode = mode;
        csr.commit();
    }
    pub fn set_trap_base(trap: u32) {
        let mut csr = STVEC::read();
        csr.base = trap;
        csr.commit();
    }
}
