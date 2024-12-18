use crate::format::{Format, ISA, O};

impl Format for O {
    const MIN_LEN: usize = 1;
    const MAX_LEN: usize = 4;
}

impl From<O> for Vec<u8> {
    fn from(value: O) -> Self {
        let x_bytes = value.off0.to_le_bytes();
        let x_len = value.off0.len();
        x_bytes[..x_len as usize].to_vec()
    }
}

impl TryFrom<&[u8]> for O {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::MIN_LEN {
            anyhow::bail!(
                "Insufficient bytes for O format, expected at least {}",
                Self::MIN_LEN
            );
        }

        // Get length capped at 4 bytes
        let x_len = bytes.len().min(4);

        // Extract offset
        let mut x_bytes = [0u8; 4];
        x_bytes[..x_len].copy_from_slice(&bytes[..x_len]);
        let x = u32::from_le_bytes(x_bytes);

        Ok(O { off0: x })
    }
}

#[test]
fn test_o_encoding() {
    let bytes = vec![3];
    let decoded = O::try_from(bytes.as_ref()).expect("Failed to decode");
    let encoded: Vec<u8> = decoded.into();
    assert_eq!(encoded, bytes);
}
