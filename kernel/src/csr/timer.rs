const MTIME_COMP_HI: *mut u32 = 0x8000100C as *mut u32;
const MTIME_COMP_LO: *mut u32 = 0x80001008 as *mut u32;

const MTIME_HI: *mut u32 = 0x80001004 as *mut u32;
const MTIME_LO: *mut u32 = 0x80001000 as *mut u32;

const CLOCK: u64 = 240 * 1000 * 1000;

#[derive(Copy, Clone)]
pub struct MicroSeccond(pub u64);

fn ms2clk(x: MicroSeccond) -> u64 {
    CLOCK * x.0 / 1000 * 1000
}

impl MicroSeccond {
    pub fn new(x: u64) -> MicroSeccond {
        MicroSeccond(x)
    }
}

pub fn set_interval(ns: MicroSeccond) {
    let clk = ms2clk(ns);
    let current = read_mtime();
    write_mtime_comp(current + clk);
}

#[allow(dead_code)]
fn read_mtime_comp() -> u64 {
    unsafe { ({ *MTIME_COMP_HI } as u64) << 32 | ({ *MTIME_COMP_LO } as u64) }
}

fn read_mtime() -> u64 {
    unsafe { ({ *MTIME_HI } as u64) << 32 | ({ *MTIME_LO } as u64) }
}

fn write_mtime_comp(x: u64) {
    let hi = (x >> 32) as u32;
    let lo = (x & 0xffffffff) as u32;
    unsafe {
        *MTIME_COMP_HI = hi;
        *MTIME_COMP_LO = lo;
    }
}

#[allow(dead_code)]
fn write_mtime(x: u64) {
    let hi = (x >> 32) as u32;
    let lo = (x & 0xffffffff) as u32;
    unsafe {
        *MTIME_HI = hi;
        *MTIME_LO = lo;
    }
}
