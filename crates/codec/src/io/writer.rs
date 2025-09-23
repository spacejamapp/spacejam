//! Writer for binary formats

use crate::{Vec, compact::vlen};

/// Writer for binary formats
pub trait Writer {
    /// Write a variable length integer
    fn write_var(&mut self, value: u32);

    /// Write a 32-bit integer
    fn write_u32(&mut self, value: u32);

    /// Write a 24-bit integer
    fn write_u24(&mut self, value: u32);

    /// Write a 16-bit integer
    fn write_u16(&mut self, value: u16);
}

impl Writer for Vec<u8> {
    fn write_var(&mut self, value: u32) {
        self.extend_from_slice(&vlen::encode(value as u64));
    }

    fn write_u32(&mut self, value: u32) {
        self.extend_from_slice(&value.to_le_bytes());
    }

    fn write_u24(&mut self, value: u32) {
        self.extend_from_slice(&value.to_le_bytes()[0..3]);
    }

    fn write_u16(&mut self, value: u16) {
        self.extend_from_slice(&value.to_le_bytes());
    }
}
