use uint::construct_uint;

construct_uint! {
    pub struct U256(4);
}

construct_uint! {
    pub struct U128(2);
}


pub trait UnsafeMathTrait {
    /// Returns ceil (x / y)
    /// Division by 0 throws a panic, and must be checked externally
    ///
    /// In Solidity dividing by 0 results in 0, not an exception.
    ///
    fn div_rounding_up(x: Self, y: Self) -> Self;
}

impl UnsafeMathTrait for u64 {
    fn div_rounding_up(x: Self, y: Self) -> Self {
        x / y + ((x % y > 0) as u64)
    }
}

impl UnsafeMathTrait for U128 {
    fn div_rounding_up(x: Self, y: Self) -> Self {
        x / y + U128::from((x % y > U128::default()) as u8)
    }
}

impl UnsafeMathTrait for U256 {
    fn div_rounding_up(x: Self, y: Self) -> Self {
        x / y + U256::from((x % y > U256::default()) as u8)
    }
}
