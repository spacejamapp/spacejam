//! JAMCodec based on the parity scale codec

mod de;
mod error;
mod ser;

pub use {
    de::{visitor::FixedBytesVisitor, Deserializer},
    error::{Error, Result},
    ser::Serializer,
};

/// Trait for types that can be encoded and decoded using JAMCodec
pub trait JamCodec: serde::Serialize + serde::de::DeserializeOwned {
    fn encode(&self) -> anyhow::Result<Vec<u8>> {
        encode(&self).map_err(Into::into)
    }

    fn decode(value: &[u8]) -> anyhow::Result<Self> {
        decode(value).map_err(Into::into)
    }
}

impl<T: serde::Serialize + serde::de::DeserializeOwned> JamCodec for T {}

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

/// Encode a value to a byte vector
pub fn encode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut serializer = Serializer::default();
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

/// Decode a value from a byte vector
pub fn decode<'de, T: serde::Deserialize<'de>>(value: &'de [u8]) -> Result<T> {
    let mut deserializer: Deserializer<'de> = Deserializer::new(value);
    T::deserialize(&mut deserializer)
}
