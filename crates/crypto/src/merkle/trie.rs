//! A space-efficient trie for storing key-value pairs.

use crate::blake2b;

/// Compute the Merkle root of a set of key-value pairs.
pub fn merkle(kvs: &[([u8; 32], Vec<u8>)], i: usize) -> [u8; 32] {
    if kvs.is_empty() {
        return [0u8; 32];
    }

    if kvs.len() == 1 {
        return blake2b(&leaf(kvs[0].0, &kvs[0].1));
    }

    let mut l = Vec::new();
    let mut r = Vec::new();
    for (k, v) in kvs {
        if bit(k, i) {
            r.push((*k, v.clone()));
        } else {
            l.push((*k, v.clone()));
        }
    }
    let encoded = branch(merkle(&l, i + 1), merkle(&r, i + 1));
    blake2b(&encoded)
}

fn branch(l: [u8; 32], r: [u8; 32]) -> [u8; 64] {
    let mut result = [0u8; 64];
    result[0] = l[0] & 0xfe;
    result[1..32].copy_from_slice(&l[1..]);
    result[32..].copy_from_slice(&r);
    result
}

fn leaf(k: [u8; 32], v: &[u8]) -> [u8; 64] {
    let mut buf = vec![0];
    buf.extend_from_slice(&k[..k.len() - 1]);

    if v.len() <= 32 {
        buf[0] = 0b01 | (v.len() << 2) as u8;
        buf.extend_from_slice(v);
        buf.resize(64, 0);
    } else {
        buf[0] = 0b11;
        buf.extend_from_slice(&blake2b(v));
    }

    let mut result = [0u8; 64];
    result.copy_from_slice(&buf[..64]);
    result
}

fn bit(k: &[u8], i: usize) -> bool {
    (k[i >> 3] & (1 << (i & 7))) != 0
}
