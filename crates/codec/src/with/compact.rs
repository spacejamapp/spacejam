//! utils for serde with compact

use crate::{visitor::VlenBytesVisitor, Numeric};

/// Serialize type with compact encoding.
pub fn serialize<S: serde::ser::Serializer, T: Numeric>(
    value: &T,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    let encoded = value.compact_encode();
    println!("compact: {:?}", encoded);
    serializer.serialize_bytes(&encoded)
}

/// Deserialize fixed byte array that larger than 32 bytes.
pub fn deserialize<'de, D: serde::de::Deserializer<'de>, T: Numeric>(
    deserializer: D,
) -> std::result::Result<T, D::Error> {
    let bytes = deserializer.deserialize_byte_buf(VlenBytesVisitor)?;
    Ok(T::compact_decode(&bytes))
}
