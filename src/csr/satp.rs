use utils;

use csr::{CSRRead, CSRWrite};

pub struct SATP {
    pub paging_on: bool,
    pub ppn: u32,
}

impl CSRRead for SATP {
    fn read_csr() -> u32 {
        let result: u32;
        unsafe {
            asm!("csrrs $0, satp, x0"
                : "=&r"(result));
        }
        result
    }
    fn from_u32(x: u32) -> SATP {
        SATP {
            paging_on: utils::bit_range(x, 31, 32) == 1,
            ppn: utils::bit_range(x, 0, 22),
        }
    }
}
impl CSRWrite for SATP {
    fn read_and_write(val: u32) -> u32 {
        let result: u32;

        unsafe {
            asm!("csrrw $0, satp, $1"
                : "=&r"(result)
                :   "r"(val));
        }
        result
    }
    fn write_csr(val: u32) {
        unsafe {
            asm!("csrrw x0, satp, $0"
                :
                : "r"(val));
        }
    }

    fn bit_set(bitvec: u32) {
        unsafe {
            asm!("csrrs x0, satp, $0"
                :
                : "r"(bitvec));
        }
    }

    fn bit_clear(bitvec: u32) {
        unsafe {
            asm!("csrrc x0, satp, $0"
                :
                : "r"(bitvec));
        }
    }
    fn to_u32(&self) -> u32 {
        let paging_on = if self.paging_on { 1 } else { 0 };
        (paging_on << 31) | self.ppn
    }
}

impl SATP {
    pub fn write(paging_on: bool, ppn: u32) {
        SATP { paging_on, ppn }.commit();
    }
    pub fn enable_paging() {
        let v = SATP {
            paging_on: true,
            ppn: 0,
        }
        .to_u32();
        SATP::bit_set(v);
    }
    pub fn disable_paging() {
        let v = SATP {
            paging_on: true,
            ppn: 0,
        }
        .to_u32();
        SATP::bit_clear(v);
    }
    pub fn set_ppn(ppn: u32) {
        let mut satp = SATP::read();
        satp.ppn = ppn;
        satp.commit();
    }
    pub fn read_ppn() -> u32 {
        let satp = SATP::read();
        satp.ppn
    }
}
