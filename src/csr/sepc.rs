use csr::{CSRRead, CSRWrite};
use paging;

pub struct SEPC {
    addr: paging::VirtAddr,
}

impl CSRRead for SEPC {
    fn read_csr() -> u32 {
        let result: u32;
        unsafe {
            asm!("csrrs $0, stvec, x0"
                : "=&r"(result));
        }
        result
    }
    fn from_u32(x: u32) -> SEPC {
        SEPC {
            addr: paging::VirtAddr::new(x),
        }
    }
}
impl CSRWrite for SEPC {
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
        self.addr.to_u32()
    }
}
