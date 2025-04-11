//! Fisher-Yates shuffle
#![cfg(feature = "shuffle")]

use crate::blake2b;

/// Equihash 329
pub fn eq329(s: &[u32], r: &[u32]) -> Vec<u32> {
    if s.is_empty() {
        return Default::default();
    }

    let l = s.len();
    let index = r[0] as usize % l;

    let mut s_post: Vec<u32> = s.to_vec();
    s_post[index] = s[l - 1];

    [[s[index]].to_vec(), eq329(&s_post[..l - 1], &r[1..])].concat()
}

/// Equihash 331
pub fn eq331(s: &[u32], h: [u8; 32]) -> Vec<u32> {
    let len = s.len();
    let r = compute_q(h, len as u32);
    eq329(s, &r)
}

/// Compute the q vector for a given hash and length.
fn compute_q(h: [u8; 32], l: u32) -> Vec<u32> {
    let mut result = vec![];
    for i in 0..l {
        let preimage = [h.to_vec(), (i / 8).to_le_bytes().to_vec()].concat();
        let offset = (4 * i % 32) as usize;

        let mut int = [0u8; 4];
        int.copy_from_slice(&blake2b(&preimage)[offset..offset + 4]);
        result.push(u32::from_le_bytes(int));
    }

    result
}
