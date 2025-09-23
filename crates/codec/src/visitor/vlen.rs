//! Variable length byte array visitor
//!
//! TODO: this visitor should be removed in the next optimization.

#[cfg(feature = "std")]
use crate::{compact::vlen, Vec};
use core::fmt;
use serde::de;

/// Visitor for variable-length numbers.
pub struct VlenBytesVisitor;

impl de::Visitor<'_> for VlenBytesVisitor {
    type Value = u64;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a variable length prefixed byte array")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
        Ok(v)
    }

    #[cfg(feature = "std")]
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E> {
        let (value, _) = vlen::decode_from(&v);
        Ok(value)
    }
}
