pub trait U128Bits {
    fn to_upper(self) -> u64;
    fn to_lower(self) -> u64;
    fn from_upper_lower(upper: u64, lower: u64) -> u128;
}

impl U128Bits for u128 {
    /// breaks out the upper half of a u128 mask
    fn to_upper(self) -> u64 {
        (self >> 64) as u64
    }

    /// breaks out the lower block of a u128 mask
    fn to_lower(self) -> u64 {
        (self & u64::MAX as u128) as u64
    }

    /// assembles a u128 bit mask from two u64 blocks
    fn from_upper_lower(upper: u64, lower: u64) -> u128 {
        ((upper as u128) << 64) | (lower as u128)
    }
}