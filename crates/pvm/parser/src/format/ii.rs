use crate::format::{Format, II, ISA};

impl Format for II {
    const MIN_LEN: usize = 2;
    const MAX_LEN: usize = 8;
}

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

impl TryFrom<&[u8]> for II {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> anyhow::Result<Self> {
        if bytes.len() < Self::MIN_LEN {
            anyhow::bail!("Insufficient bytes");
        }

        // Get l_X from first byte
        let x_len = (bytes[0] % 8).min(4);

        // Calculate l_Y based on remaining length
        let remaining = bytes.len() - (1 + x_len as usize);
        let y_len = remaining.min(4);

        // Extract first immediate
        let mid = 1 + x_len as usize;
        let (mut x_bytes, mut y_bytes) = ([0u8; 4], [0u8; 4]);
        x_bytes[..x_len as usize].copy_from_slice(&bytes[1..mid]);
        let x = u32::from_le_bytes(x_bytes);

        // Extract second immediate
        if y_len > 0 {
            y_bytes[..y_len].copy_from_slice(&bytes[mid..mid + y_len]);
        }
        let y = u32::from_le_bytes(y_bytes);

        Ok(II { imm0: x, imm1: y })
    }
}

#[test]
fn test_store_imm_u8_decode() {
    let bytes = [3, 0, 0, 2, 18];
    let decoded = II::try_from(bytes.as_ref()).expect("Failed to decode");
    let encoded: Vec<u8> = decoded.into();
    assert_eq!(encoded, bytes);
}
