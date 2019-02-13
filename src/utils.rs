// get [lb, ub) range of value
pub fn bit_range(x: u32, lb: u32, ub: u32) -> u32 {
    let y = x << (32 - ub);
    let y = y >> (32 - ub) + lb;
    y
}

pub fn round_up(x: u64, modulo: u64) -> u64 {
    x - 1 + modulo - (x - 1) % modulo
}

#[test]
fn test_bit_range() {
    assert_eq!(bit_range(0b1010101, 1, 4), 0b010);
    assert_eq!(bit_range(0b1010101, 0, 4), 0b0101);
    assert_eq!(bit_range(0b1010101, 1, 5), 0b1010);
    assert_eq!(bit_range(0b1010101, 0, 32), 0b1010101);
}
