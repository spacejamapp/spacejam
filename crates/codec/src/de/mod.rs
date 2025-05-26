//! JAMCodec deserialization implementation

use crate::{compact::vlen, Error, Result};
use serde::de::{self, Visitor};

pub mod access;

/// Deserializer for JAMCodec
pub struct Deserializer<'de> {
    input: &'de [u8],
    index: usize,
}

impl<'de> Deserializer<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        Self { input, index: 0 }
    }

    fn peek_byte(&self) -> Result<u8> {
        self.input.get(self.index).copied().ok_or_else(|| {
            anyhow::anyhow!("Failed to peek bytes, EOF: index: {}", self.index).into()
        })
    }

    /// Get the next byte from the input.
    pub fn next_byte(&mut self) -> Result<u8> {
        let byte = self.peek_byte()?;
        self.index += 1;
        Ok(byte)
    }

    /// Get the next bytes from the input.
    pub fn next_bytes(&mut self, len: usize) -> Result<&'de [u8]> {
        if self.index + len > self.input.len() {
            return Err(anyhow::anyhow!("EOF: index: {}, len: {}", self.index, len).into());
        }
        let bytes = &self.input[self.index..self.index + len];
        self.index += len;
        Ok(bytes)
    }
}

impl<'de> de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.next_byte()? != 0)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.next_byte()? as i8)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.next_byte()?;
        visitor.visit_u8(value)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.next_bytes(2)?;
        visitor.visit_i16(i16::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid i16"))?,
        ))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.next_bytes(2)?;
        visitor.visit_u16(u16::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid u16"))?,
        ))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.next_bytes(4)?;
        visitor.visit_i32(i32::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid i32"))?,
        ))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.next_bytes(4)?;
        visitor.visit_u32(u32::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid u32"))?,
        ))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.next_bytes(8)?;
        visitor.visit_i64(i64::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid i64"))?,
        ))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.next_bytes(8)?;
        visitor.visit_u64(u64::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid u64"))?,
        ))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.next_bytes(4)?;
        visitor.visit_f32(f32::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid f32"))?,
        ))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.next_bytes(8)?;
        visitor.visit_f64(f64::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid f64"))?,
        ))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_char(self.next_byte()? as char)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.next_byte()? as usize;
        let bytes = self.next_bytes(len)?;
        let s = std::str::from_utf8(bytes).map_err(|_| anyhow::anyhow!("invalid utf-8"))?;
        visitor.visit_borrowed_str(s)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // NOTE: this is only used for the compact decoding for `Vec<u8>`
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let length = vlen::decode_from_de(self)?;
        let bytes = self.next_bytes(length as usize)?;
        visitor.visit_borrowed_bytes(bytes)
    }

    /// NOTE: this is only used for the compact decoding for numeric types
    ///
    /// TODO: waiting for using compact form for all numbers in JAM.
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let prefix = self.peek_byte()?;
        if prefix < 0x80 {
            let data = self.next_byte()?;
            visitor.visit_bytes(&[data])
        } else {
            self.deserialize_bytes(visitor)
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.next_byte()? {
            0 => visitor.visit_none(),
            1 => visitor.visit_some(self),
            _ => Err(anyhow::anyhow!("invalid option").into()),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = vlen::decode_from_de(self)? as usize;
        visitor.visit_seq(access::SeqAccess::new(self, len))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(access::SeqAccess::new(self, len))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(access::SeqAccess::new(self, len))
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(anyhow::anyhow!("map is not supported").into())
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(access::SeqAccess::new(self, _fields.len()))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let variant = self.next_byte()?;
        visitor.visit_enum(access::EnumAccess::new(self, variant))
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(anyhow::anyhow!("As bytecode format, identifier is not supported").into())
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(access::SeqAccess::new(self, 0))
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}
