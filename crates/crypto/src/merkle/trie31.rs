//! A space-efficient trie for storing key-value pairs.
//! Implements binary Patricia Merkle Trie as described in the graypaper appendix D.

use crate::blake2b;

const PARALLEL_THRESHOLD: usize = 64;

/// Compute the Merkle root of a set of key-value pairs. (D.6)
pub fn trie(kvs: &[([u8; 31], &[u8])]) -> [u8; 32] {
    let mut buf = kvs.to_vec();
    merkle(&mut buf, 0)
}

/// Compute the Merkle root of a set of key-value pairs with specified depth. (D.6)
fn merkle(kvs: &mut [([u8; 31], &[u8])], depth: usize) -> [u8; 32] {
    if kvs.is_empty() {
        return [0; 32];
    }

    if kvs.len() == 1 {
        let (k, v) = kvs[0];
        return blake2b(&leaf(k, v));
    }

    // In-place partition: entries with bit=0 (left) before entries with bit=1 (right)
    let len = kvs.len();
    let mid = partition(kvs, depth);
    let (left, right) = kvs.split_at_mut(mid);
    let (l_hash, r_hash) = if len >= PARALLEL_THRESHOLD {
        rayon::join(
            || merkle(left, depth + 1),
            || merkle(right, depth + 1),
        )
    } else {
        (merkle(left, depth + 1), merkle(right, depth + 1))
    };
    blake2b(&branch(l_hash, r_hash))
}

/// Partition `kvs` in-place so that entries with bit 0 at `depth` come first.
/// Returns the index of the first "right" (bit=1) entry.
fn partition(kvs: &mut [([u8; 31], &[u8])], depth: usize) -> usize {
    let mut left_end = 0;
    for i in 0..kvs.len() {
        if !bit(&kvs[i].0, depth) {
            kvs.swap(i, left_end);
            left_end += 1;
        }
    }
    left_end
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
