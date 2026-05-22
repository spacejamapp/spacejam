//! Binary Patricia Merkle Trie (graypaper appendix D).

use crate::blake2b;

const PARALLEL_THRESHOLD: usize = 64;

/// Merkle root of a sorted key-value set (GP D.6).
pub fn trie(kvs: &[([u8; 31], &[u8])]) -> [u8; 32] {
    let mut buf = kvs.to_vec();
    merkle(&mut buf, 0)
}

fn merkle(kvs: &mut [([u8; 31], &[u8])], depth: usize) -> [u8; 32] {
    if kvs.is_empty() {
        return [0; 32];
    }
    if kvs.len() == 1 {
        let (k, v) = kvs[0];
        return blake2b(&leaf(k, v));
    }

    let len = kvs.len();
    let mid = partition(kvs, depth);
    let (left, right) = kvs.split_at_mut(mid);
    let (l_hash, r_hash) = if len >= PARALLEL_THRESHOLD {
        rayon::join(|| merkle(left, depth + 1), || merkle(right, depth + 1))
    } else {
        (merkle(left, depth + 1), merkle(right, depth + 1))
    };
    blake2b(&branch(l_hash, r_hash))
}

/// Partition index for an already-ordered key slice — linear scan, preserves order.
pub fn split_at_bit(keys: &[[u8; 31]], depth: usize) -> usize {
    keys.iter()
        .position(|k| bit(k, depth))
        .unwrap_or(keys.len())
}

/// Recover `(left, right)` child slots from a branch payload and its stored
/// child addresses; `None` if the empty-side encoding doesn't match the count.
pub fn split_branch_children<T: Copy>(
    data: &[u8],
    children: &[T],
) -> Option<(Option<T>, Option<T>)> {
    let l_empty = data[0] == 0 && data[1..32].iter().all(|&b| b == 0);
    let r_empty = data[32..64].iter().all(|&b| b == 0);
    match (l_empty, r_empty, children.len()) {
        (false, false, 2) => Some((Some(children[0]), Some(children[1]))),
        (true, false, 1) => Some((None, Some(children[0]))),
        (false, true, 1) => Some((Some(children[0]), None)),
        (true, true, 0) => Some((None, None)),
        _ => None,
    }
}

/// In-place partition of `kvs` so bit-0 entries at `depth` come first; returns the split index.
pub fn partition(kvs: &mut [([u8; 31], &[u8])], depth: usize) -> usize {
    let mut left_end = 0;
    for i in 0..kvs.len() {
        if !bit(&kvs[i].0, depth) {
            kvs.swap(i, left_end);
            left_end += 1;
        }
    }
    left_end
}

/// True when the payload's high bit of byte 0 is set (leaf marker).
pub fn is_leaf(data: &[u8]) -> bool {
    data.first().is_some_and(|b| b & 0x80 != 0)
}

/// Branch encoding (GP D.6).
pub fn branch(l: [u8; 32], r: [u8; 32]) -> [u8; 64] {
    let mut encoded = [0u8; 64];
    encoded[0] = l[0] & 0x7F;
    encoded[1..32].copy_from_slice(&l[1..]);
    encoded[32..64].copy_from_slice(&r);
    encoded
}

/// Leaf encoding (GP D.6).
pub fn leaf(k: [u8; 31], v: &[u8]) -> [u8; 64] {
    let mut encoded = [0u8; 64];
    if v.len() <= 32 {
        encoded[0] = 0x80 | (v.len() as u8 & 0x3F);
        encoded[1..32].copy_from_slice(&k);
        encoded[32..(32 + v.len())].copy_from_slice(v);
    } else {
        encoded[0] = 0xC0;
        encoded[1..32].copy_from_slice(&k);
        encoded[32..64].copy_from_slice(&blake2b(v));
    }
    encoded
}

/// Bit at `depth` (MSB-first) of a 248-bit key.
pub fn bit(k: &[u8; 31], depth: usize) -> bool {
    let byte_idx = depth / 8;
    let bit_idx = 7 - (depth % 8);
    (k[byte_idx] & (1 << bit_idx)) != 0
}
