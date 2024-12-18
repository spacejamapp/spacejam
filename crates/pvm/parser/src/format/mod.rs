//! The format of the PVM instructions.

include!(concat!(env!("OUT_DIR"), "/format.rs"));

mod i;
mod ii;
mod o;
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

    /// Read the value from the bytes.
    fn read(bytes: &[u8]) -> Self;

    /// Get the bytes of the encoding.
    fn bytes(&self) -> Vec<u8>;
}

impl ISA for u32 {
    fn is_empty(&self) -> bool {
        *self == 0
    }

    fn len(&self) -> u32 {
        if *self == 0 {
            return 1;
        }

        ((32 - self.leading_zeros() + 7) / 8).min(4)
    }

    fn read(bytes: &[u8]) -> Self {
        if bytes.is_empty() {
            return 0;
        }

        let x_len = bytes.len().min(4) as usize;
        let mut x_bytes = [0u8; 4];
        x_bytes[..x_len].copy_from_slice(&bytes[..x_len]);
        sign_extend(u32::from_le_bytes(x_bytes), x_len)
    }

    fn bytes(&self) -> Vec<u8> {
        let bytes = self.to_le_bytes().to_vec();
        let len = self.len();
        bytes[..len as usize].to_vec()
    }
}

/// Sign extend a value to 32 bits.
fn sign_extend(x: u32, n: usize) -> u32 {
    // Check the sign bit (most significant bit of the n-byte value)
    let msb = 1 << (8 * n - 1); // Position of the sign bit
    if x & msb != 0 {
        // If the sign bit is set, perform sign extension
        x | (!0 << (8 * n)) // Fill higher bits with 1s
    } else {
        // If the sign bit is not set, return x as is
        x
    }
}
