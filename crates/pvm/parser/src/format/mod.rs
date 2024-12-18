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

/// Shared trait for all formats.
pub trait Format: Sized {
    /// Minimum length of bytes required for this format
    const MIN_LEN: usize;

    /// Maximum length of bytes required for this format
    const MAX_LEN: usize;
}

/// Encoding for immediate and offset values on ISA.
pub trait ISA: Sized {
    /// Whether the value is empty.
    fn is_empty(&self) -> bool;

    /// The length of the encoding in bytes.
    fn len(&self) -> Self;

    /// Read the value from the bytes.
    fn read(bytes: &[u8]) -> anyhow::Result<Self>;

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

    fn read(bytes: &[u8]) -> anyhow::Result<Self> {
        if bytes.is_empty() {
            anyhow::bail!("No bytes provided");
        }

        let x_len = bytes.len().min(4) as usize;
        let mut x_bytes = [0u8; 4];
        x_bytes[..x_len].copy_from_slice(&bytes[..x_len]);
        Ok(u32::from_le_bytes(x_bytes))
    }

    fn bytes(&self) -> Vec<u8> {
        let bytes = self.to_le_bytes().to_vec();
        let len = self.len();
        bytes[..len as usize].to_vec()
    }
}
