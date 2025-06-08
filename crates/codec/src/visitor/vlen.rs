//! Variable length byte array visitor
//!
//! TODO: this visitor should be removed in the next optimization.

use core::fmt;
use serde::de;

/// Visitor for variable-length numbers.
pub struct VlenBytesVisitor;

impl<'de> de::Visitor<'de> for VlenBytesVisitor {
    type Value = u64;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a variable length prefixed byte array")
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v as u64)
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v as u64)
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v as u64)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v)
    }
}
