//! JAMCodec serialization implementation

use crate::{compact::vlen, Error, Result};
use serde::ser;

/// Serializer for JAMCodec
#[derive(Default)]
pub struct Serializer {
    /// Output buffer
    pub output: Vec<u8>,
}

impl ser::Serializer for &mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        if v {
            self.output.push(1);
        } else {
            self.output.push(0);
        }
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.output.extend_from_slice(&[v as u8]);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.output.push(v);
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.output.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.output.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.output.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.output.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.output.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.output.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.output.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.output.extend_from_slice(v.as_bytes());
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.output.extend_from_slice(v);
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.output.push(v as u8);
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.output.push(0);
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ser::Serialize + ?Sized,
    {
        self.output.push(1);
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Err(anyhow::anyhow!("Unit not supported").into())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        if variant_index > 0xff {
            return Err(anyhow::anyhow!("Variant index too large").into());
        }
        self.serialize_u8(variant_index as u8)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<()>
    where
        T: ser::Serialize + ?Sized,
    {
        Err(anyhow::anyhow!("Newtype struct not supported").into())
    }

    fn collect_map<K, V, I>(self, iter: I) -> Result<()>
    where
        K: ser::Serialize,
        V: ser::Serialize,
        I: IntoIterator<Item = (K, V)>,
    {
        let iter = iter.into_iter();
        let len = iter.size_hint().0;
        let length = vlen::encode(len as u64);
        self.output.extend_from_slice(&length);

        for (key, value) in iter {
            key.serialize(&mut *self)?;
            value.serialize(&mut *self)?;
        }
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        if let Some(len) = len {
            let length = vlen::encode(len as u64);
            self.output.extend_from_slice(&length);
        }
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(anyhow::anyhow!("Tuple struct not supported").into())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(anyhow::anyhow!("Tuple variant not supported").into())
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        println!("serialize_map: {len:?}");
        if let Some(len) = len {
            let length = vlen::encode(len as u64);
            self.output.extend_from_slice(&length);
        }
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(anyhow::anyhow!("Struct variant not supported").into())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_newtype_variant<T: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()> {
        if _variant_index > 0xff {
            return Err(anyhow::anyhow!("Variant index too large").into());
        }
        self.serialize_u8(_variant_index as u8)?;
        value.serialize(self)
    }

    fn collect_str<T>(self, _value: &T) -> Result<()>
    where
        T: ?Sized + std::fmt::Display,
    {
        Err(anyhow::anyhow!("Str not supported").into())
    }

    fn serialize_i128(self, v: i128) -> Result<()> {
        self.output.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }

    fn serialize_u128(self, v: u128) -> Result<()> {
        self.output.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl ser::SerializeSeq for &mut Serializer {
    type Ok = ();
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeTuple for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeTupleStruct for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        Err(anyhow::anyhow!("Tuple struct not supported").into())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeTupleVariant for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        Err(anyhow::anyhow!("Tuple variant not supported").into())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeMap for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        Ok(())
        // key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeStruct for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeStructVariant for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        Err(anyhow::anyhow!("Struct variant not supported").into())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}
