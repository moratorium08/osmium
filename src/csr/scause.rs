use crate::trap;
use csr::CSRRead;

pub struct SCAUSE {
    pub trap: trap::Trap,
}

impl CSRRead for SCAUSE {
    fn read_csr() -> u32 {
        let result: u32;
        unsafe {
            asm!("csrrs $0, scause, x0"
                : "=&r"(result));
        }
        result
    }
    fn from_u32(x: u32) -> SCAUSE {
        SCAUSE {
            trap: trap::Trap::from_u32(x).expect("failed to parse scause"),
        }
    }
}
