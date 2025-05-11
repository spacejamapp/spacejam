//! Serialize and deserialize variable length byte arrays.

use crate::compact::vlen;
use core::fmt;
use serde::de;

/// Serialize fixed byte array that larger than 32 bytes.
pub fn serialize<S: serde::ser::Serializer, T: AsRef<[u8]>>(
    value: &T,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    let len = vlen::encode(value.as_ref().len() as u64);
    let mut output = len;
    output.extend_from_slice(value.as_ref());
    serializer.serialize_bytes(&output)
}

/// Deserialize fixed byte array that larger than 32 bytes.
pub fn deserialize<'de, D: serde::de::Deserializer<'de>, T: TryFrom<Vec<u8>>>(
    deserializer: D,
) -> std::result::Result<T, D::Error> {
    let bytes = deserializer.deserialize_bytes(VarBytesVisitor)?;
    T::try_from(bytes)
        .map_err(|_| de::Error::custom("Failed to deserialize bytes with variable length"))
}

// Add this new visitor for Vec<u8>
pub struct VarBytesVisitor;

impl<'de> de::Visitor<'de> for VarBytesVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a byte array")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut bytes = Vec::new();
        while let Some(byte) = seq.next_element()? {
            bytes.push(byte);
            if bytes.len() == 9 {
                break;
            }
        }

        // get the prefix and the byte length of the prefix
        let mut output = Vec::new();
        let (length, size) = vlen::decode_from(&bytes);
        output.extend_from_slice(&bytes[size..9]);

        // complete the rest of the byte array
        while let Some(byte) = seq.next_element()? {
            output.push(byte);
            if output.len() == length as usize {
                break;
            }
        }

        Ok(output)
    }
}

// Add this implementation for deserializing Vec<u8> directly
impl<'de> de::DeserializeSeed<'de> for VarBytesVisitor {
    type Value = Vec<u8>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_tuple(0, self)
    }
}
