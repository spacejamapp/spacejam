//! Serialize and deserialize fixed byte array that larger than 32 bytes.

use crate::FixedBytesVisitor;

/// Serialize fixed byte array that larger than 32 bytes.
pub fn serialize<S: serde::ser::Serializer, T: AsRef<[u8]>>(
    value: &T,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    serializer.serialize_bytes(value.as_ref())
}

/// Deserialize fixed byte array that larger than 32 bytes.
pub fn deserialize<'de, D: serde::de::Deserializer<'de>, T: TryFrom<Vec<u8>>>(
    deserializer: D, 
) -> std::result::Result<T, D::Error> {
    deserializer.deserialize_tuple(core::mem::size_of::<T>(), FixedBytesVisitor::<T>::new())
}
