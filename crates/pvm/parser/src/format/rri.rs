use crate::format::{ISA, RRI};

impl From<RRI> for Vec<u8> {
    fn from(value: RRI) -> Self {
        let mut bytes = Vec::with_capacity(5);
        // Two registers packed into first byte
        bytes.push(((value.reg1 % 16) << 4) | (value.reg0 % 16));

        // Encode immediate
        let x_len = value.imm0.len();
        let x_bytes = value.imm0.to_le_bytes();
        bytes.extend_from_slice(&x_bytes[..x_len as usize]);

        bytes
    }
}

impl TryFrom<&[u8]> for RRI {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.is_empty() {
            anyhow::bail!("No bytes provided");
        }

        // Extract registers from first byte
        let reg0 = (bytes[0] & 0x0f).min(12);
        let reg1 = (bytes[0] >> 4).min(12);

        // Get immediate length
        let x_len = (bytes.len() - 1).min(4);

        if x_len == 0 {
            anyhow::bail!("No immediate bytes");
        }

        // Extract immediate
        let mut x_bytes = [0u8; 4];
        x_bytes[..x_len].copy_from_slice(&bytes[1..1 + x_len]);
        let x = u32::from_le_bytes(x_bytes);

        Ok(RRI {
            reg0,
            reg1,
            imm0: x,
        })
    }
}
