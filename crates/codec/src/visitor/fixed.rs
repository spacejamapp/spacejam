//! Fixed-size byte array visitor

use crate::{format, Vec};
use core::fmt;
use serde::de::{self, Error};

/// Visitor for fixed-size byte arrays
#[derive(Default)]
pub struct FixedBytesVisitor<T: TryFrom<Vec<u8>>> {
    _marker: core::marker::PhantomData<T>,
}

impl<T: TryFrom<Vec<u8>>> FixedBytesVisitor<T> {
    pub fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

impl<'v, T: TryFrom<Vec<u8>>> de::Visitor<'v> for FixedBytesVisitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(&format!(
            "a fixed-size byte array of {} bytes",
            core::mem::size_of::<T>()
        ))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'v>,
    {
        let mut bytes = Vec::with_capacity(core::mem::size_of::<T>());
        for _ in 0..core::mem::size_of::<T>() {
            bytes.push(seq.next_element()?.unwrap_or_default());
        }
        T::try_from(bytes).map_err(|_| A::Error::custom("invalid bytes: {bytes:?}"))
    }
}

impl<'v, T: TryFrom<Vec<u8>>> de::DeserializeSeed<'v> for FixedBytesVisitor<T> {
    type Value = T;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'v>,
    {
        deserializer.deserialize_tuple(core::mem::size_of::<T>(), self)
    }
}
