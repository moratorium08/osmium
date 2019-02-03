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

fn read_and_write(addr: u16, val: u32) -> u32 {
    let result: u32;

    unsafe {
        asm!("csrrs $0, $1, $2"
            : "=&r"(result)
            : "r"(addr),
                "r"(val));
    }
    result
}

fn write(addr: u16, val: u32) {
    unsafe {
        asm!("csrrw x0, $0, $1"
            :
            : "r"(addr),
              "r"(val));
    }
}

fn bit_set(addr: u16, bitvec: u32) {
    unsafe {
        asm!("csrrw x0, $0, $1"
            :
            : "r"(addr),
              "r"(bitvec));
    }
}

fn bit_clear(addr: u16, bitvec: u32) {
    unsafe {
        asm!("csrrc x0, $0, $1"
            :
            : "r"(addr),
              "r"(bitvec));
    }

}

const SATP_ADDR: u16 = 0x180;
pub struct SATP {
    pub paging_on: bool,
    pub ppn: u32
}

impl SATP {
    fn to_u32(&self) -> u32 {
        let paging_on = if self.paging_on {1} else {0};
        (paging_on << 31) | self.ppn
    }
    pub fn read() -> SATP {
        let v = read(SATP_ADDR);
        SATP{paging_on: utils::bit_range(v, 31, 32) == 1,
             ppn: utils::bit_range(v, 0, 22)}
    }
    pub fn commit(&self) {
        let v = self.to_u32();
        write(SATP_ADDR, v);
    }
    pub fn write(paging_on: bool, ppn: u32) {
        SATP{paging_on, ppn}.commit();
    }
    pub fn enable_paging() {
        let v = SATP{paging_on: true, ppn: 0}.to_u32();
        bit_set(SATP_ADDR, v);
    }
    pub fn disable_paging() {
        let v = SATP{paging_on: true, ppn: 0}.to_u32();
        bit_clear(SATP_ADDR, v);
    }
    pub fn set_ppn(ppn: u32) {
        let mut satp = SATP::read();
        satp.ppn = ppn;
        satp.commit();
    }
}