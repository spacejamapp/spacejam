//! Reader for binary formats

use crate::{compact::vlen, Cow};

/// Reader for binary formats
pub trait Reader {
    /// Read a variable length integer
    fn read_var(&mut self) -> Option<u32>;

    /// Read a 32-bit integer
    fn read_u32(&mut self) -> Option<u32>;

    /// Read a 24-bit integer
    fn read_u24(&mut self) -> Option<u32>;

    /// Read a 16-bit integer
    fn read_u16(&mut self) -> Option<u16>;

    /// Read a 8-bit integer
    fn read_u8(&mut self) -> Option<u8>;
}

impl Reader for &[u8] {
    fn read_var(&mut self) -> Option<u32> {
        let (value, length) = vlen::decode_from(self);
        *self = &self[length..];
        Some(value as u32)
    }

    fn read_u32(&mut self) -> Option<u32> {
        if self.len() < 4 {
            return None;
        }

        let value = u32::from_le_bytes([self[0], self[1], self[2], self[3]]);
        *self = &self[4..];
        Some(value)
    }

    fn read_u24(&mut self) -> Option<u32> {
        if self.len() < 3 {
            return None;
        }

        let value = u32::from_le_bytes([self[0], self[1], self[2], 0]);
        *self = &self[3..];
        Some(value)
    }

    fn read_u16(&mut self) -> Option<u16> {
        if self.len() < 2 {
            return None;
        }

        let value = u16::from_le_bytes([self[0], self[1]]);
        *self = &self[2..];
        Some(value)
    }

    fn read_u8(&mut self) -> Option<u8> {
        if self.is_empty() {
            return None;
        }

        let value = self[0];
        *self = &self[1..];
        Some(value)
    }
}

/// Read a borrowed slice of bytes from the reader.
pub fn read<'a>(bytes: &mut &'a [u8], length: u32) -> Option<Cow<'a, [u8]>> {
    let length = length as usize;
    let slice = bytes.get(..length)?;
    *bytes = &bytes[length..];
    Some(Cow::Borrowed(slice))
}
