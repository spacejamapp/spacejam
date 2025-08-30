//! Value type for host functions

use cranelift::prelude::{types, Type};

/// Value type for host functions
#[repr(u8)]
pub enum Value {
    /// 8-bit signed integer
    I8,
    /// 16-bit signed integer
    I16,
    /// 32-bit signed integer
    I32,
    /// 64-bit signed integer
    I64,
    /// 8-bit unsigned integer
    U8,
    /// 16-bit unsigned integer
    U16,
    /// 32-bit unsigned integer
    U32,
    /// 64-bit unsigned integer
    U64,
}

impl Value {
    /// Convert the value to a slice of bytes
    pub fn as_bytes(&self, data: i64) -> Vec<u8> {
        match self {
            Value::I8 | Value::U8 => vec![data as u8],
            Value::I16 | Value::U16 => vec![data as u8, (data >> 8) as u8],
            Value::I32 | Value::U32 => vec![
                data as u8,
                (data >> 8) as u8,
                (data >> 16) as u8,
                (data >> 24) as u8,
            ],
            Value::I64 | Value::U64 => vec![
                data as u8,
                (data >> 8) as u8,
                (data >> 16) as u8,
                (data >> 24) as u8,
                (data >> 32) as u8,
                (data >> 40) as u8,
                (data >> 48) as u8,
                (data >> 56) as u8,
            ],
        }
    }

    /// Get the number of bytes for the value
    pub fn bytes(&self) -> usize {
        match self {
            Value::I8 | Value::U8 => 1,
            Value::I16 | Value::U16 => 2,
            Value::I32 | Value::U32 => 4,
            Value::I64 | Value::U64 => 8,
        }
    }

    /// Convert the value to a u64
    pub fn as_u64(&self, bytes: &[u8]) -> anyhow::Result<u64> {
        Ok(match self {
            Value::I8 | Value::U8 => u8::from_le_bytes([bytes[0]]) as u64,
            Value::I16 | Value::U16 => u16::from_le_bytes([bytes[0], bytes[1]]) as u64,
            Value::I32 | Value::U32 => {
                u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as u64
            }
            Value::I64 | Value::U64 => u64::from_le_bytes(
                bytes[..8]
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("invalid bytes"))?,
            ),
        })
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        match value {
            0 => Value::I8,
            1 => Value::I16,
            2 => Value::I32,
            3 => Value::I64,
            4 => Value::U8,
            5 => Value::U16,
            6 => Value::U32,
            7 => Value::U64,
            _ => panic!("invalid value: {value}"),
        }
    }
}

impl From<Type> for Value {
    fn from(ty: Type) -> Self {
        match ty {
            types::I8 => Value::I8,
            types::I16 => Value::I16,
            types::I32 => Value::I32,
            types::I64 => Value::I64,
            _ => panic!("invalid type: {ty}"),
        }
    }
}
