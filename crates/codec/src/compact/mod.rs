//! Number encoding and decoding

pub mod num;
pub mod vlen;

use crate::visitor::VlenBytesVisitor;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
pub use {
    num::Numeric,
    vlen::{decode, decode_from, encode},
};

/// A compact number.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub struct Compact<T: Serialize>(pub T);

impl<T: Serialize> Compact<T> {
    /// Create a new compact number
    pub fn new(value: T) -> Self {
        Compact(value)
    }
}

impl<T: Serialize + Clone> Compact<T> {
    /// Create a new compact number
    pub fn cloned(&self) -> T {
        self.0.clone()
    }
}

impl<T: Default + Serialize> Default for Compact<T> {
    fn default() -> Self {
        Compact(T::default())
    }
}

impl<T: Serialize> Deref for Compact<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Serialize> DerefMut for Compact<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Serialize> From<T> for Compact<T> {
    fn from(value: T) -> Self {
        Compact(value)
    }
}

mod compact_num {
    use super::*;

    /// Serialize a compact number
    impl<T: Numeric> Serialize for Compact<T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_bytes(&self.0.compact_encode())
        }
    }

    /// Deserialize a compact number
    impl<'de, T: Numeric> Deserialize<'de> for Compact<T> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let bytes = deserializer.deserialize_byte_buf(VlenBytesVisitor)?;
            Ok(Compact(T::compact_decode(&bytes)))
        }
    }
}
