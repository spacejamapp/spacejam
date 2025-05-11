//! Vector visitor

use serde::de;
use std::fmt;

// Add this new visitor for Vec<u8>
pub struct VecU8Visitor;

impl<'de> de::Visitor<'de> for VecU8Visitor {
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
        }
        Ok(bytes)
    }
}

// Add this implementation for deserializing Vec<u8> directly
impl<'de> de::DeserializeSeed<'de> for VecU8Visitor {
    type Value = Vec<u8>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_tuple(0, self)
    }
}
