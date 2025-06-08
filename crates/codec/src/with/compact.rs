//! Serialize and deserialize fixed byte array that larger than 32 bytes.

use crate::{visitor::VlenBytesVisitor, Numeric};

/// Serialize compact number.
pub fn serialize<S: serde::ser::Serializer, T: Numeric>(
    value: &T,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    serializer.serialize_bytes(&value.compact_encode())
}

/// Deserialize compact number.
pub fn deserialize<'de, D: serde::de::Deserializer<'de>, T: Numeric>(
    deserializer: D,
) -> std::result::Result<T, D::Error> {
    let value = deserializer.deserialize_byte_buf(VlenBytesVisitor)?;
    Ok(T::from_u64(value))

    // match T::LENGTH {
    //     1 => {
    //         let value = deserializer.deserialize_u8(VlenBytesVisitor)?;
    //         Ok(T::from_u64(value))
    //     }
    //     2 => {
    //         let value = deserializer.deserialize_u16(VlenBytesVisitor)?;
    //         Ok(T::from_u64(value))
    //     }
    //     4 => {
    //         let value = deserializer.deserialize_u32(VlenBytesVisitor)?;
    //         Ok(T::from_u64(value))
    //     }
    //     8 => {
    //         let value = deserializer.deserialize_u64(VlenBytesVisitor)?;
    //         Ok(T::from_u64(value))
    //     }
    //     _ => {
    //         let value = deserializer.deserialize_byte_buf(VlenBytesVisitor)?;
    //         Ok(T::from_u64(value))
    //     }
    // }
}
