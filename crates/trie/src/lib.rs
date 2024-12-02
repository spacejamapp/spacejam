//! A space-efficient trie for storing key-value pairs.

use blake2::{digest::consts::U32, Blake2b, Digest};

/// Compute the Merkle root of a set of key-value pairs.
pub fn merkle(kvs: &[(Vec<u8>, Vec<u8>)], i: usize) -> [u8; 32] {
    if kvs.is_empty() {
        return [0u8; 32];
    }
    if kvs.len() == 1 {
        return leaf(&kvs[0].0, &kvs[0].1);
    }
    let mut l = Vec::new();
    let mut r = Vec::new();
    for (k, v) in kvs {
        if bit(k, i) {
            r.push((k.clone(), v.clone()));
        } else {
            l.push((k.clone(), v.clone()));
        }
    }
    let left = merkle(&l, i + 1);
    let right = merkle(&r, i + 1);
    let encoded = branch(left, right);
    hash(&encoded)
}

fn hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Blake2b::<U32>::new();
    hasher.update(data);
    hasher.finalize().into()
}

fn branch(l: [u8; 32], r: [u8; 32]) -> [u8; 64] {
    let mut head = l[0];
    head &= 0xfe;

    let mut result = [0u8; 64];
    result[0] = head;
    result[1..].copy_from_slice(&l[1..]);
    result[33..].copy_from_slice(&r);
    result
}

fn leaf(k: &[u8], v: &[u8]) -> [u8; 32] {
    let mut buf = vec![0];
    buf.extend_from_slice(&k[..k.len() - 1]);

    if v.len() <= 32 {
        buf[0] = 0b01 | (v.len() << 2) as u8;
        buf.extend_from_slice(v);
    } else {
        buf[0] = 0b11;
        buf.extend_from_slice(&hash(v));
    }

    let mut result = [0u8; 32];
    result.copy_from_slice(&buf);
    result
}

fn bit(k: &[u8], i: usize) -> bool {
    (k[i >> 3] & (1 << (i & 7))) != 0
}
