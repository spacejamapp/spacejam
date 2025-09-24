//! primitive json conversions

use crate::{Json, String};
use anyhow::Result;

macro_rules! impl_primitive {
    ($($ty:ty),*) => {
        $(
            impl Json<$ty> for $ty {
                fn to_json(self) -> $ty {
                    self
                }

                fn from_json(json: $ty) -> Result<Self> {
                    Ok(json)
                }
            }
        )*
    };
}

impl_primitive!(
    u8,
    u16,
    u32,
    u64,
    u128,
    i8,
    i16,
    i32,
    i64,
    i128,
    usize,
    bool,
    (),
    String
);
