use csr::{CSRRead, CSRWrite};
use utils;

pub struct SIE {
    pub mtimer: bool,
}

impl CSRRead for SIE {
    fn read_csr() -> u32 {
        let result: u32;
        unsafe {
            asm!("csrrs $0, sie, x0"
                : "=&r"(result));
        }
        result
    }
    fn from_u32(x: u32) -> SIE {
        SIE {
            mtimer: utils::bit_range(x, 5, 6) == 1,
        }
    }
}
impl CSRWrite for SIE {
    fn read_and_write(val: u32) -> u32 {
        let result: u32;

        unsafe {
            asm!("csrrw $0, sie, $1"
                : "=&r"(result)
                :   "r"(val));
        }
        result
    }
    fn write_csr(val: u32) {
        unsafe {
            asm!("csrrw x0, sie, $0"
                :
                : "r"(val));
        }
    }

    fn bit_set(bitvec: u32) {
        unsafe {
            asm!("csrrs x0, sie, $0"
                :
                : "r"(bitvec));
        }
    }

    fn bit_clear(bitvec: u32) {
        unsafe {
            asm!("csrrc x0, sie, $0"
                :
                : "r"(bitvec));
        }
    }
    fn to_u32(&self) -> u32 {
        let mtimer = if self.mtimer { 1 << 5 } else { 0 };
        mtimer
    }
}

impl SIE {
    pub fn mtimer_on() {
        let v = SIE { mtimer: true }.to_u32();
        SIE::bit_set(v);
    }
}
