//! Transaction related utilities

use score::{BandersnatchPublic, OpaqueHash, extrinsic::TicketsOrKeys};

/// Create a fallback series
pub fn fallback(ring: Vec<BandersnatchPublic>, entropy: OpaqueHash) -> TicketsOrKeys {
    let mut keys = [BandersnatchPublic::default(); score::EPOCH_LENGTH as usize];
    for i in 0..score::EPOCH_LENGTH {
        let input = [entropy.as_slice(), &i.to_le_bytes()].concat();
        let hash = crypto::blake2b(&input);
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&hash[0..4]);
        let index = u32::from_le_bytes(bytes) % (ring.len() as u32);
        keys[i as usize] = ring[index as usize];
    }

    TicketsOrKeys::Keys(keys)
}
