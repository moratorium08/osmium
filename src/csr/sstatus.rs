use csr::{CSRRead, CSRWrite};

pub struct SSTATUS {
    pub spie: bool,
    pub sie: bool,
}

impl CSRRead for SSTATUS {
    fn read_csr() -> u32 {
        let result: u32;
        unsafe {
            asm!("csrrs $0, sstatus, x0"
                : "=&r"(result));
        }
        result
    }
    fn from_u32(x: u32) -> SSTATUS {
        SSTATUS {
            spie: utils::bit_range(x, 5, 6) == 1,
            sie: utils::bit_range(x, 1, 2) == 1,
        }
    }
}
impl CSRWrite for SSTATUS {
    fn read_and_write(val: u32) -> u32 {
        let result: u32;

        unsafe {
            asm!("csrrw $0, sstatus, $1"
                : "=&r"(result)
                :   "r"(val));
        }
        result
    }
    fn write_csr(val: u32) {
        unsafe {
            asm!("csrrw x0, sstatus, $0"
                :
                : "r"(val));
        }
    }

    fn bit_set(bitvec: u32) {
        unsafe {
            asm!("csrrs x0, sstatus, $0"
                :
                : "r"(bitvec));
        }
    }

    fn bit_clear(bitvec: u32) {
        unsafe {
            asm!("csrrc x0, sstatus, $0"
                :
                : "r"(bitvec));
        }
    }
    fn to_u32(&self) -> u32 {
        let spie = if self.spie { 1 << 5 } else { 0 };
        let sie = if self.sie { 1 << 1 } else { 0 };
        spie | sie
    }
}

impl SSTATUS {
    pub fn spie_on() {
        let v = SSTATUS {
            spie: true,
            sie: false,
        }
        .to_u32();
        SSTATUS::bit_set(v);
    }
}
