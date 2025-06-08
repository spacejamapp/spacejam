//! utils for serde with compact

use crate::{visitor::VlenBytesVisitor, Numeric};

/// Serialize type with compact encoding.
pub fn serialize<S: serde::ser::Serializer, T: Numeric>(
    value: &T,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    serializer.serialize_bytes(&value.compact_encode())
}

/// Deserialize fixed byte array that larger than 32 bytes.
pub fn deserialize<'de, D: serde::de::Deserializer<'de>, T: Numeric>(
    deserializer: D,
) -> std::result::Result<T, D::Error> {
    let bytes = deserializer.deserialize_any(VlenBytesVisitor)?;
    Ok(T::compact_decode(&bytes))
}
