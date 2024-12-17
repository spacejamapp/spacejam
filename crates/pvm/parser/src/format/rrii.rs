use crate::format::{Format, ISA, RRII};

impl Format for RRII {
    const MIN_LEN: usize = 3;
    const MAX_LEN: usize = 9;
}

impl From<RRII> for Vec<u8> {
    fn from(value: RRII) -> Self {
        let mut bytes = Vec::with_capacity(6);
        // Two registers packed into first byte
        bytes.push(((value.reg1 % 16) << 4) | (value.reg0 % 16));

        // Calculate minimum bytes needed for immediates
        let x_len = value.imm0.len();
        let y_len = value.imm1.len();

        // Length byte for first immediate
        bytes.push(x_len as u8 % 8);

        // Encode first immediate
        let x_bytes = value.imm0.to_le_bytes();
        bytes.extend_from_slice(&x_bytes[..x_len as usize]);

        // Encode second immediate
        let y_bytes = value.imm1.to_le_bytes();
        bytes.extend_from_slice(&y_bytes[..y_len as usize]);

        bytes
    }
}

impl TryFrom<&[u8]> for RRII {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::MIN_LEN {
            anyhow::bail!("Insufficient bytes");
        }

        // Extract registers from first byte
        let reg0 = (bytes[0] & 0x0f).min(12);
        let reg1 = (bytes[0] >> 4).min(12);

        // Get l_X from second byte
        let x_len = (bytes[1] % 8).min(4);

        // Calculate l_Y based on remaining length
        let remaining = bytes.len() - (2 + x_len as usize);
        let y_len = remaining.min(4);

        // Extract first immediate
        let mut x_bytes = [0u8; 4];
        x_bytes[..x_len as usize].copy_from_slice(&bytes[2..2 + x_len as usize]);
        let x = u32::from_le_bytes(x_bytes);

        // Extract second immediate
        let mut y_bytes = [0u8; 4];
        if y_len > 0 {
            y_bytes[..y_len]
                .copy_from_slice(&bytes[2 + x_len as usize..2 + x_len as usize + y_len]);
        }
        let y = u32::from_le_bytes(y_bytes);

        Ok(RRII {
            reg0,
            reg1,
            imm0: x,
            imm1: y,
        })
    }
}
