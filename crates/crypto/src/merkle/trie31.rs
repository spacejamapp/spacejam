//! A space-efficient trie for storing key-value pairs.
//! Implements binary Patricia Merkle Trie as described in the graypaper appendix D.

use crate::blake2b;

/// Compute the Merkle root of a set of key-value pairs. (D.6)
pub fn trie(kvs: &[([u8; 31], Vec<u8>)]) -> [u8; 32] {
    merkle(kvs, 0)
}

/// Compute the Merkle root of a set of key-value pairs with specified depth. (D.6)
pub fn merkle(kvs: &[([u8; 31], Vec<u8>)], depth: usize) -> [u8; 32] {
    if kvs.is_empty() {
        return [0; 32];
    }

    if kvs.len() == 1 {
        let (k, ref v) = &kvs[0];
        return blake2b(&leaf(*k, v));
    }

    let (mut left, mut right) = (Vec::new(), Vec::new());
    for (k, v) in kvs {
        if bit(k, depth) {
            right.push((*k, v.clone()));
        } else {
            left.push((*k, v.clone()));
        }
    }

    // Recursive calls with incremented depth
    let l_hash = merkle(&left, depth + 1);
    let r_hash = merkle(&right, depth + 1);

    // According to D.6, M(d) = H(bits^{-1}(B(M(l), M(r))))
    blake2b(&branch(l_hash, r_hash))
}

/// Branch encoding
fn branch(l: [u8; 32], r: [u8; 32]) -> [u8; 64] {
    let mut encoded = [0u8; 64];
    encoded[0] = l[0] & 0x7F; // 0b01111111
    encoded[1..32].copy_from_slice(&l[1..]);
    encoded[32..64].copy_from_slice(&r);
    encoded
}

/// Leaf encoding
fn leaf(k: [u8; 31], v: &[u8]) -> [u8; 64] {
    let mut encoded = [0u8; 64];

    if v.len() <= 32 {
        // 0x80 = 0b10000000, 0x3F = 0b00111111
        encoded[0] = 0x80 | (v.len() as u8 & 0x3F);
        encoded[1..32].copy_from_slice(&k);
        encoded[32..(32 + v.len())].copy_from_slice(v);

        if v.len() < 32 {
            encoded[(32 + v.len())..64].fill(0);
        }
    } else {
        encoded[0] = 0xC0; // 0b11000000
        encoded[1..32].copy_from_slice(&k);
        encoded[32..64].copy_from_slice(&blake2b(v));
    }

    encoded
}

/// Get the bit at the specified position in the key
/// For 31-byte keys, we have 248 bits (31*8)
fn bit(k: &[u8; 31], depth: usize) -> bool {
    let byte_idx = depth / 8;
    let bit_idx = 7 - (depth % 8);
    (k[byte_idx] & (1 << bit_idx)) != 0
}
