//! Number encoding and decoding

pub mod num;
mod vlen;

use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
pub use {
    num::Numeric,
    vlen::{decode, decode_from, encode},
};

/// A compact number.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Compact<T: Numeric + Default>(T);

impl<T: Numeric> Compact<T> {
    /// Create a new compact number
    pub fn cloned(&self) -> T {
        self.0
    }
}

impl<T: Numeric> Default for Compact<T> {
    fn default() -> Self {
        Compact(T::default())
    }
}

impl<T: Numeric> Deref for Compact<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Numeric> DerefMut for Compact<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Numeric> Serialize for Compact<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0.compact_encode())
    }
}

impl<'de, T: Numeric> Deserialize<'de> for Compact<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = <Vec<u8>>::deserialize(deserializer)?;
        Ok(Compact(T::compact_decode(&bytes)))
    }
}

impl<T: Numeric> From<T> for Compact<T> {
    fn from(value: T) -> Self {
        Compact(value)
    }
}
