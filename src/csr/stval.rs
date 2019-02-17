use csr::CSRRead;

pub struct STVAL {
    pub val: u32,
}

impl CSRRead for STVAL {
    fn read_csr() -> u32 {
        let result: u32;
        unsafe {
            asm!("csrrs $0, stval, x0"
                : "=&r"(result));
        }
        result
    }
    fn from_u32(val: u32) -> STVAL {
        STVAL { val }
    }
}
