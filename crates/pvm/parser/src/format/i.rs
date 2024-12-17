use crate::format::{Format, I, ISA};

impl Format for I {
    const MIN_LEN: usize = 1;
    const MAX_LEN: usize = 4;
}

impl From<I> for Vec<u8> {
    fn from(value: I) -> Self {
        let x_bytes = value.imm0.to_le_bytes();
        let x_len = value.imm0.len();
        x_bytes[..x_len as usize].to_vec()
    }
}

impl TryFrom<&[u8]> for I {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::MIN_LEN {
            anyhow::bail!("Insufficient bytes, expected at least {}", Self::MIN_LEN);
        }

        // Get length capped at 4 bytes
        let x_len = bytes.len().min(4);

        // Extract offset
        let mut x_bytes = [0u8; 4];
        x_bytes[..x_len].copy_from_slice(&bytes[..x_len]);
        let x = u32::from_le_bytes(x_bytes);
        Ok(I { imm0: x })
    }
}

#[test]
fn test_i_encoding() {
    let bytes = vec![3];
    let decoded = I::try_from(bytes.as_ref()).expect("Failed to decode");
    let encoded: Vec<u8> = decoded.into();
    assert_eq!(encoded, bytes);
}
