//! State key constructor

use crate::misc::OpaqueHash;

macro_rules! to_key {
    ($key:expr) => {
        [
            $key, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ]
    };
}

/// C(1) - The authorization pools (α)
pub const AUTHORIZATION_POOLS: OpaqueHash = to_key!(1);

/// C(2) - The authorization queue (φ)
pub const AUTHORIZATION_QUEUE: OpaqueHash = to_key!(2);

/// C(3) - The recent blocks (β)
pub const RECENT_BLOCKS: OpaqueHash = to_key!(3);

/// C(4) - State concerning Safrole (γ)
pub const SAFROLE: OpaqueHash = to_key!(4);

/// C(5) - Past judgments on work-reports and validators (ψ)
pub const JUDGEMENTS: OpaqueHash = to_key!(5);

/// C(6) - The entropy accumulator and epochal randomness (η)
pub const ENTROPY: OpaqueHash = to_key!(6);

/// C(7) - The next validators (ι)
pub const NEXT_VALIDATORS: OpaqueHash = to_key!(7);

/// C(8) - The current validators (κ)
pub const CURRENT_VALIDATORS: OpaqueHash = to_key!(8);

/// C(9) - The previous validators (λ)
pub const PREVIOUS_VALIDATORS: OpaqueHash = to_key!(9);

/// C(10) - The pending reports, per core, which are being made available prior to
/// accumulation. (ρ)
pub const PENDING_REPORTS: OpaqueHash = to_key!(10);

/// C(11) - The current timeslot (τ)
pub const TIMESLOT: OpaqueHash = to_key!(11);

/// C(12) - The privileged service indices (χ)
pub const PRIVILEGED_SERVICE: OpaqueHash = to_key!(12);

/// C(13) - The activity statistics for the validators (π)
pub const STATISTICS: OpaqueHash = to_key!(13);

/// C(14) - The accumulation queue (θ)
pub const ACCUMULATION_QUEUE: OpaqueHash = to_key!(14);

/// C(15) - The accumulation history (ξ)
pub const ACCUMULATION_HISTORY: OpaqueHash = to_key!(15);

/// A trait for state key construction
trait StateKey {
    /// The key of the state
    fn key(&self) -> OpaqueHash;
}

impl StateKey for u8 {
    fn key(&self) -> OpaqueHash {
        let mut key = [0u8; 32];
        key[0] = *self;
        key
    }
}

// for service indices
impl StateKey for (u8, u32) {
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
impl StateKey for (u32, [u8; 32]) {
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

/// Service account keys
pub mod account {
    use super::StateKey;
    use crate::misc::OpaqueHash;

    /// C(255, s) - The service account state ((s -> a) δ)
    pub fn state(service: u32) -> OpaqueHash {
        (255, service).key()
    }

    /// C(s, [(2^32 - 1), k0...28]) maybe verdict? ((s ->a ->k ->v) δ)
    ///
    /// from storage dictionary s
    pub fn storage(service: u32, k: OpaqueHash) -> OpaqueHash {
        let mut key = [0u8; 32];
        key[..4].copy_from_slice(&(u32::MAX - 1).to_le_bytes());
        key[4..].copy_from_slice(&k[..28]);
        (service, key).key()
    }

    /// C(s, [(2^32 - 2), k0...28]) maybe preimage? ((s ->(a ->h) ->p) δ)
    pub fn preimage(service: u32, h: OpaqueHash) -> OpaqueHash {
        let mut key = [0u8; 32];
        key[..4].copy_from_slice(&(u32::MAX - 2).to_le_bytes());
        key[4..].copy_from_slice(&h[1..29]);
        (service, key).key()
    }

    /// C(s, [(2^32 - 3), k0...28]) maybe lookup? ((s ->a ->h ->l) δ)
    ///
    /// TODO: maybe embed this function to the account service
    pub fn lookup(service: u32, lookup: u32, h: OpaqueHash) -> OpaqueHash {
        let mut key = [0; 32];
        let hashed = crypto::blake2b(&h);
        key[..4].copy_from_slice(&lookup.to_le_bytes());
        key[4..].copy_from_slice(&hashed[2..30]);
        (service, key).key()
    }
}
