use crate::format::{ISA, RRII};

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

impl From<&[u8]> for RRII {
    fn from(bytes: &[u8]) -> Self {
        if bytes.is_empty() || bytes == [0] {
            return Default::default();
        }

        let reg0 = (bytes[0] & 0x0f).min(12);
        let reg1 = (bytes[0] >> 4).min(12);

        if bytes.len() == 1 {
            return RRII {
                reg0,
                reg1,
                imm0: 0,
                imm1: 0,
            };
        }

        let x_len = (bytes[1] % 8).min(4);
        let mid = 2 + x_len as usize;

        RRII {
            reg0,
            reg1,
            imm0: u64::read_imm(&bytes[2..mid]),
            imm1: u64::read_imm(&bytes[mid..]),
        }
    }
}
