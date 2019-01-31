// get [lb, ub) range of value
pub fn bit_range(mut value: u32, lb: u8, ub: u8) -> u32 {
    value <<= 32 - lb;
    value >>= 31 - (lb - ub);
    value
}