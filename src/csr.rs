use utils;


fn read(addr: u16) -> u32 {
    let result: u32;
    unsafe {
        asm!("csrrs $0, $1, x0"
            : "=&r"(result)
            : "r"(addr));
    }
    result
}

trait CSR {
    fn to_u32(&self) -> u32;
    fn addr(&self) -> u16;
    fn update(&mut self, val: u32);

    fn read_and_write(&mut self) {
        let addr = self.addr();
        let val = self.to_u32();
        let result: u32;

        unsafe {
            asm!("csrrs $0, $1, $2"
                : "=&r"(result)
                : "r"(addr),
                  "r"(val));
        }
    }

    fn write(&self) {
        let result: u32;
        unsafe {
            asm!("csrrw x0, $0, $1"
                :
                : "r"(self.addr()),
                  "r"(self.to_u32()));
        }
    }

    fn bit_set(&self, bitvec: u32) {
        unsafe {
            asm!("csrrw x0, $0, $1"
                :
                : "r"(self.addr()),
                  "r"(bitvec));
        }
    }

    fn bit_clear(&self, bitvec: u32) {
        unsafe {
            asm!("csrrc x0, $0, $1"
                :
                : "r"(self.addr()),
                  "r"(bitvec));
        }

    }
}

const SATP_ADDR: u16 = 0x180;
struct SATP {
    paging_on: bool,
    ppn: u32
}

impl SATP {
    fn new(ppn: u32, paging_on: bool) -> SATP {
        SATP{ppn, paging_on}
    }
    fn read() -> SATP {
        let v = read(SATP_ADDR);
        SATP{paging_on: utils::bit_range(v, 31, 32) == 1,
             ppn: utils::bit_range(v, 0, 22)}
    }
}


impl CSR for SATP {
    fn to_u32(&self) -> u32 {
        let paging_on = if self.paging_on {1} else {0};
        (paging_on << 31) | self.ppn
    }
    
    fn addr(&self) -> u16 {
        SATP_ADDR
    }

    fn update(&mut self, val: u32) {
        self.paging_on = utils::bit_range(val, 31, 32) == 1;
        self.ppn = utils::bit_range(val, 0, 22);
    }
}