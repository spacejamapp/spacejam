//! Variable length byte array visitor
//!
//! TODO: this visitor should be removed in the next optimization.

use core::fmt;
use serde::de;

/// Visitor for variable-length byte arrays.
/// This visitor correctly handles the vlen encoded format where the length
/// is encoded as a prefix before the actual data.
///
/// FIXME: should not re-encode / decode for types.
pub struct VlenBytesVisitor;

impl<'de> de::Visitor<'de> for VlenBytesVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a variable length prefixed byte array")
    }

    fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(bytes.to_vec())
    }
}
