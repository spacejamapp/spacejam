use crate::format::{Format, ISA, RRI};

impl Format for RRI {
    const MIN_LEN: usize = 2;
    const MAX_LEN: usize = 5;
}

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
        if bytes.len() < Self::MIN_LEN {
            anyhow::bail!("Invalid length, expected at least 3 bytes");
        }

        Ok(RRI {
            reg0: (bytes[0] & 0x0f).min(12),
            reg1: (bytes[0] >> 4).min(12),
            imm0: u32::read(&bytes[1..])?,
        })
    }
}
