pub mod satp;
pub mod scause;
pub mod sepc;
pub mod sstatus;
pub mod stval;
pub mod stvec;

pub trait CSRWrite {
    fn read_and_write(val: u32) -> u32;
    fn write_csr(val: u32);
    fn bit_set(bitvec: u32);
    fn bit_clear(bitvec: u32);
    fn to_u32(&self) -> u32;
    fn commit(&self) {
        let v = self.to_u32();
        Self::write_csr(v);
    }
}

pub trait CSRRead: Sized {
    fn read_csr() -> u32;
    fn from_u32(x: u32) -> Self;
    fn read() -> Self {
        Self::from_u32(Self::read_csr())
    }
}
