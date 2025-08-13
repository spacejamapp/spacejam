//! State key constructor

use crate::TrieKey;

macro_rules! to_key {
    ($key:expr) => {
        [
            $key, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]
    };
}

/// C(1) - The authorization pools (α)
pub const AUTHORIZATION_POOLS: TrieKey = to_key!(1);

/// C(2) - The authorization queue (φ)
pub const AUTHORIZATION_QUEUE: TrieKey = to_key!(2);

/// C(3) - The recent blocks (β)
pub const RECENT_BLOCKS: TrieKey = to_key!(3);

/// C(4) - State concerning Safrole (γ)
pub const SAFROLE: TrieKey = to_key!(4);

/// C(5) - Past judgments (disputes) on work-reports and validators (ψ)
pub const DISPUTES: TrieKey = to_key!(5);

/// C(6) - The entropy accumulator and epochal randomness (η)
pub const ENTROPY: TrieKey = to_key!(6);

/// C(7) - The next validators (ι)
pub const NEXT_VALIDATORS: TrieKey = to_key!(7);

/// C(8) - The current validators (κ)
pub const CURRENT_VALIDATORS: TrieKey = to_key!(8);

/// C(9) - The previous validators (λ)
pub const PREVIOUS_VALIDATORS: TrieKey = to_key!(9);

/// C(10) - The pending reports, per core, which are being made available prior to
/// accumulation. (ρ)
pub const PENDING_REPORTS: TrieKey = to_key!(10);

/// C(11) - The current timeslot (τ)
pub const TIMESLOT: TrieKey = to_key!(11);

/// C(12) - The privileged service indices (χ)
pub const PRIVILEGED_SERVICE: TrieKey = to_key!(12);

/// C(13) - The activity statistics for the validators (π)
pub const STATISTICS: TrieKey = to_key!(13);

/// C(14) - The accumulation queue (θ)
pub const ACCUMULATION_QUEUE: TrieKey = to_key!(14);

/// C(15) - The accumulation history (ξ)
pub const ACCUMULATION_HISTORY: TrieKey = to_key!(15);

/// C(16) - The accumulation logs?
pub const ACCUMULATION_LOGS: TrieKey = to_key!(16);

/// The prefix of account storage (u32::MAX - 1)
pub const ACCOUNT_STORAGE_PREFIX: [u8; 4] = [255, 255, 255, 255];

/// The prefix of account preimage (u32::MAX - 2)
pub const ACCOUNT_PREIMAGE_PREFIX: [u8; 4] = [254, 255, 255, 255];

/// The constant keys
pub const CONSTANT_KEYS: [TrieKey; 15] = [
    AUTHORIZATION_POOLS,
    AUTHORIZATION_QUEUE,
    RECENT_BLOCKS,
    SAFROLE,
    DISPUTES,
    ENTROPY,
    NEXT_VALIDATORS,
    CURRENT_VALIDATORS,
    PREVIOUS_VALIDATORS,
    PENDING_REPORTS,
    TIMESLOT,
    PRIVILEGED_SERVICE,
    STATISTICS,
    ACCUMULATION_QUEUE,
    ACCUMULATION_HISTORY,
];

/// A trait for state key construction
pub trait StorageKeyEncode {
    /// The key of the state
    fn key(&self) -> TrieKey;
}

impl StorageKeyEncode for u8 {
    fn key(&self) -> TrieKey {
        let mut key = [0u8; 31];
        key[0] = *self;
        key
    }
}

// for service indices
impl StorageKeyEncode for (u8, u32) {
    fn key(&self) -> TrieKey {
        let mut key = [0u8; 31];
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
//
// FIXME: This seems not correct, I forgot if it is used anywhere.
impl StorageKeyEncode for (u32, [u8; 32]) {
    fn key(&self) -> TrieKey {
        let mut key = [0u8; 31];
        let (s, h) = *self;

        // Format: [n0, h0, n1, h1, n2, h2, n3, h3, h4, h5, ..., h27]
        let mut hashp = [0; 4];
        hashp.copy_from_slice(&h[..4]);
        key[..8].copy_from_slice(&prefix(s, &hashp));
        key[8..].copy_from_slice(&h[4..27]);
        key
    }
}

/// Generate a prefix for a storage
///
/// service: [0, 2, 4, 6]
/// prefix: [1, 3, 5, 7]
pub fn prefix(service: u32, prefix: &[u8; 4]) -> [u8; 8] {
    let mut key = [0; 8];
    service
        .to_le_bytes()
        .iter()
        .zip(prefix.iter())
        .enumerate()
        .for_each(|(i, (a, b))| {
            key[i * 2] = *a;
            key[(i + 1) * 2 - 1] = *b;
        });
    key
}
