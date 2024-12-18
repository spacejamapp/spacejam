use crate::format::{Format, ISA, RI};

impl Format for RI {
    const MIN_LEN: usize = 2;
    const MAX_LEN: usize = 5;
}

impl From<RI> for Vec<u8> {
    fn from(value: RI) -> Vec<u8> {
        let x_len = value.imm0.len();
        let mut bytes = Vec::with_capacity(1 + x_len as usize);
        bytes.push(value.reg0 % 16); // Register encoded mod 16

        // Encode immediate
        let x_bytes = value.imm0.to_le_bytes();
        bytes.extend_from_slice(&x_bytes[..x_len as usize]);
        bytes
    }
}

impl TryFrom<&[u8]> for RI {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> anyhow::Result<Self> {
        if bytes == &[0] {
            return Ok(RI { reg0: 0, imm0: 0 });
        }

        // Get immediate length capped at 4 bytes
        let x_len = (bytes.len() - 1).min(4);
        if x_len == 0 {
            anyhow::bail!(
                "Insufficient bytes for RI format, expected at least {}",
                Self::MIN_LEN
            );
        }

        // Extract immediate
        let mut x_bytes = [0u8; 4];
        x_bytes[..x_len].copy_from_slice(&bytes[1..1 + x_len]);

        Ok(RI {
            reg0: bytes[0] % 16,
            imm0: u32::from_le_bytes(x_bytes),
        })
    }
}

#[test]
fn test_ri_encoding() {
    let bytes = vec![7, 0, 0, 2];
    let decoded = RI::try_from(bytes.as_ref()).expect("Failed to decode");
    assert_eq!(decoded.reg0, 7);
    assert_eq!(decoded.imm0, 0x020000);

    let encoded: Vec<u8> = decoded.into();
    assert_eq!(encoded, bytes);
}
