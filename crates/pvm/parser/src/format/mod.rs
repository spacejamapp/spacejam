//! The format of the PVM instructions.

use crate::Register;

include!(concat!(env!("OUT_DIR"), "/format.rs"));

mod i;
mod ii;
mod o;
mod rei;
mod ri;
mod rii;
mod rio;
mod rr;
mod rri;
mod rrii;
mod rro;
mod rrr;

/// Encoding for immediate and offset values on ISA.
pub trait ISA: Sized {
    /// Whether the value is empty.
    fn is_empty(&self) -> bool;

    /// The length of the encoding in bytes.
    fn len(&self) -> Self;

    /// Read the immediate value from the bytes.
    fn read_imm(bytes: &[u8]) -> Self;

    /// Get the bytes of the encoding.
    fn bytes(&self) -> Vec<u8>;

    /// Sign extends the value from n bytes to 4 bytes (32 bits)
    /// n must be in {0,1,2,3,4,8}
    fn sign_extend(&self, n: usize) -> Self;

    /// Sign extends the value in 32 bits
    fn sign_ext32(&self) -> Self;

    /// Read the value from the bytes.
    fn read(bytes: &[u8]) -> (Self, usize);
}

impl ISA for u64 {
    fn is_empty(&self) -> bool {
        *self == 0
    }

    fn len(&self) -> u64 {
        if *self == 0 {
            return 1;
        }

        (64 - self.leading_zeros()).div_ceil(8).min(8) as u64
    }

    fn read(bytes: &[u8]) -> (Self, usize) {
        if bytes.is_empty() {
            return (0, 0);
        }

        let x_len = bytes.len().min(8);
        let mut x_bytes = [0u8; 8];
        x_bytes[..x_len].copy_from_slice(&bytes[..x_len]);
        (u64::from_le_bytes(x_bytes), x_len)
    }

    fn read_imm(bytes: &[u8]) -> Self {
        let (x, x_len) = Self::read(bytes);
        x.sign_extend(x_len)
    }

    fn sign_extend(&self, n: usize) -> Self {
        if n == 0 || n == 8 {
            return *self;
        }

        // Calculate 2^(8n-1)
        let bit_pos = 8 * n - 1;
        let sign_bit = (*self >> bit_pos) & 1;

        if sign_bit == 1 {
            // If sign bit is 1, extend with 1s
            // Create mask with 1s in upper bits: (2^64 - 2^8n) in the formula
            let mask = !((1 << (8 * n)) - 1);
            *self | mask
        } else {
            // If sign bit is 0, extend with 0s
            *self
        }
    }

    // sign extend the value in 32 bits
    //
    // see also `self.sign_extend(4)`
    fn sign_ext32(&self) -> Self {
        if (*self & 0x80000000) != 0 {
            *self | 0xFFFFFFFF00000000
        } else {
            *self
        }
    }

    fn bytes(&self) -> Vec<u8> {
        let bytes = self.to_le_bytes().to_vec();
        let len = self.len();
        bytes[..len as usize].to_vec()
    }
}
