//! The I instruction format.

use crate::format::{I, ISA};

impl From<I> for Vec<u8> {
    fn from(value: I) -> Self {
        let x_bytes = value.imm0.to_le_bytes();
        let x_len = value.imm0.len();
        x_bytes[..x_len as usize].to_vec()
    }
}

impl From<&[u8]> for I {
    fn from(bytes: &[u8]) -> Self {
        if bytes == [0] || bytes.is_empty() {
            return Default::default();
        }

        I {
            imm0: u64::read_imm(&bytes[..bytes.len().min(8)]),
        }
    }
}

#[test]
fn test_i_encoding() {
    let bytes = vec![3];
    let decoded = I::from(bytes.as_ref());
    let encoded: Vec<u8> = decoded.into();
    assert_eq!(encoded, bytes);
}
