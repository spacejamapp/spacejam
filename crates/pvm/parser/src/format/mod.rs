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

    /// Read the immediate value from the bytes.
    fn read_imm(bytes: &[u8]) -> Self;

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
        read_u32(bytes).0
    }

    fn read_imm(bytes: &[u8]) -> Self {
        let (x, x_len) = read_u32(bytes);
        sign_extend(x, x_len)
    }

    fn bytes(&self) -> Vec<u8> {
        let bytes = self.to_le_bytes().to_vec();
        let len = self.len();
        bytes[..len as usize].to_vec()
    }
}

/// Sign extends a value from n bytes to 4 bytes (32 bits)
/// n must be in {0,1,2,3,4,8}
pub fn sign_extend(x: u32, n: usize) -> u32 {
    if n == 0 || n == 4 {
        return x;
    }

    // Calculate 2^(8n-1)
    let bit_pos = 8 * n - 1;
    let sign_bit = (x >> bit_pos) & 1;

    if sign_bit == 1 {
        // If sign bit is 1, extend with 1s
        // Create mask with 1s in upper bits: (2^64 - 2^8n) in the formula
        let mask = !((1 << (8 * n)) - 1);
        x | mask
    } else {
        // If sign bit is 0, extend with 0s
        x
    }
}

/// Read a u32 from the bytes.
fn read_u32(bytes: &[u8]) -> (u32, usize) {
    if bytes.is_empty() {
        return (0, 0);
    }

    let x_len = bytes.len().min(4);
    let mut x_bytes = [0u8; 4];
    x_bytes[..x_len].copy_from_slice(&bytes[..x_len]);
    (u32::from_le_bytes(x_bytes), x_len)
}
