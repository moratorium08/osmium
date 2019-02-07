// get [lb, ub) range of value
pub fn bit_range(mut value: u32, lb: u8, ub: u8) -> u32 {
    let right_shift = (31 - lb) + ub;
    if right_shift >= 32 {
        0
    } else {
        value <<= 32 - lb;
        value >>= right_shift;
        value
    }
}
