//! Serialize and deserialize fixed byte array that larger than 32 bytes.

use crate::visitor::FixedBytesVisitor;
use serde::{de, de::Error, ser, ser::SerializeSeq};

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

/// Serialize and deserialize fixed byte array that larger than 32 bytes.
pub mod array {
    use super::*;
    use std::fmt;

    /// Serialize array of fixed byte arrays
    pub fn serialize<S: ser::Serializer, T: AsRef<[u8]>>(
        value: &[T],
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(value.len()))?;
        for item in value {
            seq.serialize_element(item.as_ref())?;
        }
        seq.end()
    }

    /// Deserialize array of fixed byte arrays
    pub fn deserialize<'de, D: de::Deserializer<'de>, T: TryFrom<Vec<u8>>>(
        deserializer: D,
    ) -> std::result::Result<Vec<T>, D::Error> {
        deserializer.deserialize_seq(FixedArrayVisitor::<T>::new())
    }

    /// Visitor for arrays of fixed-size byte arrays
    #[derive(Default)]
    struct FixedArrayVisitor<T: TryFrom<Vec<u8>>> {
        _marker: std::marker::PhantomData<T>,
    }

    impl<T: TryFrom<Vec<u8>>> FixedArrayVisitor<T> {
        fn new() -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }
    }

    impl<'de, T: TryFrom<Vec<u8>>> de::Visitor<'de> for FixedArrayVisitor<T> {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence of fixed-size byte arrays")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut result = Vec::new();
            while let Some(bytes) = seq.next_element::<Vec<u8>>()? {
                let item = T::try_from(bytes)
                    .map_err(|_| A::Error::custom("failed to convert bytes to fixed array"))?;
                result.push(item);
            }
            Ok(result)
        }
    }
}
