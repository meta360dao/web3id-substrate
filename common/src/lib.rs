#![cfg_attr(not(feature = "std"), no_std)]

pub mod traits {
    /// Config to provide price of currency pairs. The price is multiplied by 1000 and rounded to make it integer
    pub trait PriceProvider {
        // NOTE: Consider returning weight when None as well

        /// Get the latest price of Dock in terms of USD. Returns the price and consumed weight in this operation
        fn get_dock_usd_price() -> Option<(u32, u64)>;

        /// Get the latest price of Dock in terms of USD but it is an optimized call so it might not get
        /// the latest price. Returns the price and consumed weight in this operation
        fn optimized_get_dock_usd_price() -> Option<(u32, u64)>;
    }
}

pub mod arith_utils {
    pub trait DivCeil: Sized {
        fn div_ceil(&self, other: Self) -> Self;

        fn checked_div_ceil(&self, other: Self) -> Option<Self>;
    }

    impl DivCeil for u32 {
        fn div_ceil(&self, other: Self) -> Self {
            let (q, r) = (self / other, self % other);

            q + (r != 0) as u32
        }

        fn checked_div_ceil(&self, other: Self) -> Option<Self> {
            let (q, r) = (self.checked_div(other)?, self.checked_rem(other)?);

            Some(q + (r != 0) as u32)
        }
    }
}
