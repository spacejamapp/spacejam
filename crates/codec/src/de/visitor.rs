//! JAMCodec deserialization visitor

use serde::de;
use std::fmt;

/// Visitor for JAMCodec deserialization
#[derive(Default)]
pub struct Visitor<T: TryFrom<Vec<u8>>> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: TryFrom<Vec<u8>>> Visitor<T> {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: TryFrom<Vec<u8>>> de::Visitor<'_> for Visitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a byte vector")
    }
}
