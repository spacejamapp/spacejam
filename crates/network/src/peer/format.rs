//! U256 implementation for the peer id
#![allow(clippy::manual_div_ceil)]

use uint::construct_uint;

// chars for peer id encoding: abcdefghijklmnopqrstuvwxyz234567
const ALPHABET: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', '2', '3', '4', '5', '6', '7',
];

construct_uint! {
    /// U256 type
    pub struct U256(4);
}

/// Encode a public key to a peer id
pub fn encode(public: &[u8; 32]) -> String {
    let mut id = String::new();
    id.push('e');

    let mut num = U256::from_little_endian(public);
    for _ in 0..52 {
        id.push(ALPHABET[(num % 32).as_usize()]);
        num /= 32;
    }

    id
}

/// Decode a peer id to a public key
pub fn decode(id: &str) -> Result<[u8; 32], anyhow::Error> {
    if !id.starts_with('e') {
        anyhow::bail!(
            "Invalid peer id prefix, expected 'e', got {:?}",
            id.chars().next()
        );
    }

    let len = id.len();
    if len != 53 {
        anyhow::bail!("Invalid peer id length, expected 53, got {len}");
    }

    let mut num = U256::zero();
    let chars: Vec<char> = id.chars().skip(1).collect();
    for c in chars.iter().rev() {
        let index = ALPHABET
            .iter()
            .position(|&a| a == *c)
            .ok_or_else(|| anyhow::anyhow!("Invalid character in peer id: {}", c))?;
        num = num * 32 + U256::from(index);
    }

    let mut public = [0u8; 32];
    public.copy_from_slice(&num.to_little_endian());
    Ok(public)
}

#[test]
fn test_encode_decode() {
    let buf =
        hex::decode("3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29").unwrap();
    let mut public = [0; 32];
    public.copy_from_slice(&buf);

    let id = encode(&public);
    assert_eq!(id, "e3r2oc62zwfj3crnuifuvsxvbtlzetk4o5qyhetkhagsc2fgl2oka");
    let decoded = decode(&id).unwrap();
    assert_eq!(public, decoded);
}
