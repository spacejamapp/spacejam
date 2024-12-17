use crate::format::{ISA, RII};

impl From<RII> for Vec<u8> {
    fn from(value: RII) -> Self {
        let mut bytes = Vec::with_capacity(5);
        // Register encoded mod 16 in first byte
        bytes.push(value.reg0 % 16);

        // Calculate minimum bytes needed for immediates
        let x_len = value.imm0.len();
        let y_len = value.imm1.len();

        // Encode first immediate
        let x_bytes = value.imm0.to_le_bytes();
        bytes.extend_from_slice(&x_bytes[..x_len as usize]);

        // Encode second immediate
        let y_bytes = value.imm1.to_le_bytes();
        bytes.extend_from_slice(&y_bytes[..y_len as usize]);

        bytes
    }
}

impl TryFrom<&[u8]> for RII {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.is_empty() {
            anyhow::bail!("No bytes provided");
        }

        // Get register index
        let reg = bytes[0] % 16;

        // Get immediate lengths
        let remaining = bytes.len() - 1;
        let x_len = remaining.min(4);
        let y_len = (remaining - x_len).min(4);

        if x_len == 0 {
            anyhow::bail!("No immediate bytes");
        }

        // Extract first immediate
        let mut x_bytes = [0u8; 4];
        x_bytes[..x_len].copy_from_slice(&bytes[1..1 + x_len]);
        let x = u32::from_le_bytes(x_bytes);

        // Extract second immediate
        let mut y_bytes = [0u8; 4];
        if y_len > 0 {
            y_bytes[..y_len].copy_from_slice(&bytes[1 + x_len..1 + x_len + y_len]);
        }
        let y = u32::from_le_bytes(y_bytes);

        Ok(RII {
            reg0: reg,
            imm0: x,
            imm1: y,
        })
    }
}
