//! The O instruction format.

use crate::format::{I, O};

impl From<O> for Vec<u8> {
    fn from(value: O) -> Self {
        I {
            imm0: value.off0 as u64,
        }
        .into()
    }
}

impl From<&[u8]> for O {
    fn from(bytes: &[u8]) -> Self {
        let i = I::from(bytes);
        O {
            off0: i.imm0 as i32,
        }
    }
}

#[test]
fn test_o_encoding() {
    let bytes = vec![3];
    let decoded = O::from(bytes.as_ref());
    let encoded: Vec<u8> = decoded.into();
    assert_eq!(encoded, bytes);
}
