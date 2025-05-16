//! Variable length byte array visitor

use core::fmt;
use serde::de;

/// Visitor for variable-length byte arrays.
/// This visitor correctly handles the vlen encoded format where the length
/// is encoded as a prefix before the actual data.
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

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        // Collect bytes into a Vec
        let mut bytes = Vec::new();
        while let Some(byte) = seq.next_element()? {
            bytes.push(byte);
        }

        // Process the bytes using the visit_bytes method
        self.visit_bytes(&bytes)
    }
}
