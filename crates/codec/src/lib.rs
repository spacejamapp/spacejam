//! JAMCodec based on the parity scale codec
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub(crate) use internal::*;
pub use {
    compact::Numeric,
    de::Deserializer,
    error::{Error, Result},
    io::{Reader, Writer},
    ser::Serializer,
    with::bytes,
};

pub mod compact;
mod de;
mod error;
mod internal;
pub mod io;
mod ser;
pub mod visitor;
mod with;

/// Trait for types that can be encoded and decoded using JAMCodec
pub trait JamCodec: serde::Serialize + serde::de::DeserializeOwned {
    /// Encode the value into a byte vector
    fn encode(&self) -> anyhow::Result<Vec<u8>> {
        encode(&self).map_err(Into::into)
    }

    /// Decode the value from a byte vector
    fn decode(value: &[u8]) -> anyhow::Result<Self> {
        decode(value).map_err(Into::into)
    }
}

impl<T: serde::Serialize + serde::de::DeserializeOwned> JamCodec for T {}

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
