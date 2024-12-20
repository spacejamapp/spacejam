//! State key constructor

use crate::misc::OpaqueHash;

/// A trait for state key construction
pub trait Key {
    /// The key of the state
    fn key(&self) -> OpaqueHash;
}

impl Key for u8 {
    fn key(&self) -> OpaqueHash {
        let mut key = [0u8; 32];
        key[0] = *self;
        key
    }
}

// for service indices
impl Key for (u8, u32) {
    fn key(&self) -> OpaqueHash {
        let mut key = [0u8; 32];
        let (i, s) = *self;
        let n = s.to_le_bytes();

        // Format: [i, n0, 0, n1, 0, n2, 0, n3, 0, 0, ...]
        key[0] = i;
        key[1] = n[0];
        key[2] = 0;
        key[3] = n[1];
        key[4] = 0;
        key[5] = n[2];
        key[6] = 0;
        key[7] = n[3];
        key[8] = 0;

        key
    }
}

// used for service account state keys
impl Key for (u32, [u8; 32]) {
    fn key(&self) -> OpaqueHash {
        let mut key = [0u8; 32];
        let (s, h) = *self;
        let n = s.to_le_bytes();

        // Format: [n0, h0, n1, h1, n2, h2, n3, h3, h4, h5, ..., h27]
        key[0] = n[0];
        key[1] = h[0];
        key[2] = n[1];
        key[3] = h[1];
        key[4] = n[2];
        key[5] = h[2];
        key[6] = n[3];
        key[7] = h[3];

        key[8..].copy_from_slice(&h[4..28]);
        key
    }
}
