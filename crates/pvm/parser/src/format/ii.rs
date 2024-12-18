use crate::format::{II, ISA};

impl From<II> for Vec<u8> {
    fn from(value: II) -> Self {
        let [x_len, y_len] = [value.imm0.len(), value.imm1.len()];

        let mut bytes = Vec::with_capacity(1 + x_len as usize + y_len as usize);
        bytes.push(x_len as u8 % 8); // l_X encoded in first byte mod 8

        // Encode immediates
        bytes.extend_from_slice(&value.imm0.to_le_bytes()[..x_len as usize]);
        bytes.extend_from_slice(&value.imm1.to_le_bytes()[..y_len as usize]);
        bytes
    }
}

impl From<&[u8]> for II {
    fn from(bytes: &[u8]) -> Self {
        if bytes == [0] || bytes.is_empty() {
            return Default::default();
        }

        let x_len = (bytes[0] % 8).min(4);
        if x_len == 0 {
            return Default::default();
        }

        let mid = 1 + x_len as usize;

        II {
            imm0: u32::read(&bytes[1..mid]),
            imm1: u32::read(&bytes[mid..]),
        }
    }
}

#[test]
fn test_store_imm_u8() {
    let bytes = [3, 0, 0, 2, 18];
    let decoded = II::from(bytes.as_ref());
    let encoded: Vec<u8> = decoded.into();
    assert_eq!(encoded, bytes);
}

#[test]
fn test_store_imm_u32() {
    let bytes = [3, 0, 0, 2, 120, 86, 52, 18];
    let decoded = II::from(bytes.as_ref());
    let encoded: Vec<u8> = decoded.into();
    assert_eq!(encoded, bytes);
}
