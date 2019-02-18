use csr::{CSRRead, CSRWrite};
use utils;

pub struct SIP {
    pub timer: bool,
}

impl CSRRead for SIP {
    fn read_csr() -> u32 {
        let result: u32;
        unsafe {
            asm!("csrrs $0, sip, x0"
                : "=&r"(result));
        }
        result
    }
    fn from_u32(x: u32) -> SIP {
        SIP {
            timer: utils::bit_range(x, 5, 6) == 1,
        }
    }
}
impl CSRWrite for SIP {
    fn read_and_write(val: u32) -> u32 {
        let result: u32;

        unsafe {
            asm!("csrrw $0, sip, $1"
                : "=&r"(result)
                :   "r"(val));
        }
        result
    }
    fn write_csr(val: u32) {
        unsafe {
            asm!("csrrw x0, sip, $0"
                :
                : "r"(val));
        }
    }

    fn bit_set(bitvec: u32) {
        unsafe {
            asm!("csrrs x0, sip, $0"
                :
                : "r"(bitvec));
        }
    }

    fn bit_clear(bitvec: u32) {
        unsafe {
            asm!("csrrc x0, sip, $0"
                :
                : "r"(bitvec));
        }
    }
    fn to_u32(&self) -> u32 {
        let timer = if self.timer { 1 << 5 } else { 0 };
        timer
    }
}

impl SIP {
    pub fn timer_off() {
        let v = SIP { timer: true }.to_u32();
        SIP::bit_clear(v);
    }
}
