use crate::trap;
use csr::CSRRead;

pub enum Trap {
    Exception(trap::Exception),
    Interruption(trap::Interruption),
}

impl Trap {
    fn from_u32(x: u32) -> Option<Trap> {
        if (x >> 31) == 1 {
            let int = trap::Interruption::from_u32(x & !(1 << 31));
            match int {
                Some(int) => Some(Trap::Interruption(int)),
                None => None,
            }
        } else {
            let exc = trap::Exception::from_u32(x & !(1 << 31));
            match exc {
                Some(int) => Some(Trap::Exception(int)),
                None => None,
            }
        }
    }
}

pub struct SCAUSE {
    pub trap: Trap,
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
            trap: Trap::from_u32(x).expect("failed to parse scause"),
        }
    }
}
