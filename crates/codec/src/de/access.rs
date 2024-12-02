use crate::{Deserializer, Error, Result};
use serde::de::{self, Deserializer as _};

/// Access for sequence
pub struct SeqAccess<'a, 'de> {
    deserializer: &'a mut Deserializer<'de>,
    len: usize,
    current: usize,
}

impl<'a, 'de> SeqAccess<'a, 'de> {
    /// Create a new sequence access
    pub fn new(deserializer: &'a mut Deserializer<'de>, len: usize) -> Self {
        SeqAccess {
            deserializer,
            len,
            current: 0,
        }
    }
}

impl<'a, 'de> de::SeqAccess<'de> for SeqAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.current >= self.len {
            return Ok(None);
        }
        self.current += 1;
        seed.deserialize(&mut *self.deserializer).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

/// Access for enum
pub struct EnumAccess<'a, 'de> {
    deserializer: &'a mut Deserializer<'de>,
    /// Variant of enum
    #[allow(dead_code)]
    pub variant: u8,
}

impl<'a, 'de> EnumAccess<'a, 'de> {
    /// Create a new enum access
    pub fn new(deserializer: &'a mut Deserializer<'de>, variant: u8) -> Self {
        EnumAccess {
            deserializer,
            variant,
        }
    }
}

impl<'a, 'de> de::EnumAccess<'de> for EnumAccess<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, _seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        Err(anyhow::anyhow!("variant seed").into())
    }
}

impl<'a, 'de> de::VariantAccess<'de> for EnumAccess<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.deserializer)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserializer.deserialize_seq(visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserializer.deserialize_struct("", _fields, visitor)
    }
}
