use crate::{Deserializer, Error, Result};
use serde::de::{self, IntoDeserializer};

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

impl<'de> de::SeqAccess<'de> for SeqAccess<'_, 'de> {
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

impl<'de> de::EnumAccess<'de> for EnumAccess<'_, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let val = seed
            .deserialize(self.variant.into_deserializer())
            .map_err(|e: Error| e)?;

        Ok((val, self))
    }
}

impl<'de> de::VariantAccess<'de> for EnumAccess<'_, 'de> {
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

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_tuple(self.deserializer, len, visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_tuple(self.deserializer, fields.len(), visitor)
    }
}

pub struct MapAccess<'a, 'de> {
    deserializer: &'a mut Deserializer<'de>,
    len: usize,
    current: usize,
}

impl<'a, 'de> MapAccess<'a, 'de> {
    /// Create a new map access
    pub fn new(deserializer: &'a mut Deserializer<'de>, len: usize) -> Self {
        MapAccess {
            deserializer,
            len,
            current: 0,
        }
    }
}

impl<'de> de::MapAccess<'de> for MapAccess<'_, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.current >= self.len {
            return Ok(None);
        }

        seed.deserialize(&mut *self.deserializer).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(&mut *self.deserializer)?;
        self.current += 1;
        Ok(value)
    }
}
