//! JAMCodec deserialization visitor

use serde::de;
use std::fmt;

/// Visitor for fixed-size byte arrays
#[derive(Default)]
pub struct FixedBytesVisitor<T: TryFrom<Vec<u8>>> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: TryFrom<Vec<u8>>> FixedBytesVisitor<T> {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: TryFrom<Vec<u8>>> de::Visitor<'_> for FixedBytesVisitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a byte vector")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::try_from(v.to_vec()).map_err(|_| E::custom("invalid bytes"))
    }
}
