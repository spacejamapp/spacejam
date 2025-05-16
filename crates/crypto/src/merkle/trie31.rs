//! A space-efficient trie for storing key-value pairs.

use crate::blake2b;

/// Compute the Merkle root of a set of key-value pairs. (D.6)
pub fn merkle(kvs: &[([u8; 31], Vec<u8>)]) -> [u8; 32] {
    if kvs.is_empty() {
        return [0; 32];
    }

    if kvs.len() == 1 {
        return blake2b(&leaf(kvs[0].0, &kvs[0].1));
    }

    let (mut left, mut right) = (Vec::new(), Vec::new());
    for &(k, ref v) in kvs {
        let first_bit = (k[0] & 0b10000000) != 0;

        let mut new_key = [0u8; 31];
        for i in 0..30 {
            new_key[i] = (k[i] << 1) | (k[i + 1] >> 7);
        }

        new_key[30] = k[30] << 1;

        if first_bit {
            right.push((new_key, v.clone()));
        } else {
            left.push((new_key, v.clone()));
        }
    }

    blake2b(&branch(merkle(&left), merkle(&right)))
}

/// Branch encoding via (D.3)
fn branch(l: [u8; 32], r: [u8; 32]) -> [u8; 64] {
    let mut encoded = [0u8; 64];
    encoded[0] = l[0] & 0b01111111;
    encoded[1..32].copy_from_slice(&l[1..]);
    encoded[32..].copy_from_slice(&r);
    encoded
}

/// Leaf encoding via (D.4)
fn leaf(k: [u8; 31], v: &[u8]) -> [u8; 64] {
    let mut encoded = [0u8; 64];
    let length = v.len();

    if length <= 32 {
        encoded[0] = 0b10000000;
        encoded[0] |= ((length as u8) & 0b00111111) << 2;
        encoded[1..32].copy_from_slice(&k);
        encoded[32..32 + length].copy_from_slice(v);
    } else {
        encoded[0] = 0b11000000;
        encoded[1..32].copy_from_slice(&k);
        encoded[32..].copy_from_slice(&blake2b(v));
    }

    encoded
}
