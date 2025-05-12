//! Serialize and deserialize variable length byte arrays.

use crate::{compact::vlen, visitor::VlenBytesVisitor};
use serde::de;

/// Serialize fixed byte array that larger than 32 bytes.
pub fn serialize<S: serde::ser::Serializer, T: AsRef<[u8]>>(
    value: &T,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    let mut output = vlen::encode(value.as_ref().len() as u64);
    output.extend_from_slice(value.as_ref());
    serializer.serialize_bytes(&output)
}

/// Deserialize variable length byte array
pub fn deserialize<'de, D: serde::de::Deserializer<'de>, T: TryFrom<Vec<u8>>>(
    deserializer: D,
) -> std::result::Result<T, D::Error> {
    let bytes = deserializer.deserialize_bytes(VlenBytesVisitor)?;
    T::try_from(bytes)
        .map_err(|_| de::Error::custom("Failed to deserialize bytes with variable length"))
}
