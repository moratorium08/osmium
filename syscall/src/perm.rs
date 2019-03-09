bitflags! {
    pub struct Perm: u32{
        const READ  = 1 << 1;
        const WRITE = 1 << 2;
        const EXEC  = 1 << 3;
    }
}
