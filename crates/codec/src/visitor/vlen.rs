//! Variable length byte array visitor

use core::fmt;
use serde::de;

// Add this new visitor for Vec<u8>
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
        // For the var module, the actual decoding of the length prefix has already been done
        // by the time we get here, so we just need to return the full byte array
        Ok(bytes.to_vec())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        // Collect all bytes
        let mut bytes = Vec::new();
        while let Some(byte) = seq.next_element()? {
            bytes.push(byte);
        }

        // No need to process the length prefix - just return the full byte array
        Ok(bytes)
    }
}

// Add this implementation for deserializing Vec<u8> directly
impl<'de> de::DeserializeSeed<'de> for VlenBytesVisitor {
    type Value = Vec<u8>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(self)
    }
}
