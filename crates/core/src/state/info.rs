//! State information

use crate::{state::key, OpaqueHash, TrieKey};

/// State key like interface
pub trait StateKeyLike {
    /// Get the state key
    fn as_state_key(&self) -> TrieKey;
}

impl StateKeyLike for OpaqueHash {
    fn as_state_key(&self) -> TrieKey {
        let mut buf = [0u8; 31];
        buf[..31].copy_from_slice(self);
        buf
    }
}

impl StateKeyLike for Vec<u8> {
    fn as_state_key(&self) -> TrieKey {
        let mut buf = [0u8; 31];
        let len = self.len().min(31);
        buf[..len].copy_from_slice(&self[..len]);
        buf
    }
}

impl StateKeyLike for &[u8] {
    fn as_state_key(&self) -> TrieKey {
        let mut buf = [0u8; 31];
        let len = self.len().min(31);
        buf[..len].copy_from_slice(&self[..len]);
        buf
    }
}

/// A key in the state pairs
pub trait StateKeyInfo {
    /// Get the information about the key
    fn info(&self) -> StateKey;
}

/// The state information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateKey {
    /// The authorization pools (α)
    AuthorizationPools,
    /// The recent blocks (β)
    RecentBlocks,
    /// The safrole (γ)
    Safrole,
    /// The accounts (δ)
    Account { service: u32, field: ServiceField },
    /// The entropy (η)
    Entropy,
    /// The validators (ι, κ, λ)
    Validators { kind: ValidatorKind },
    /// The pending reports (ρ)
    PendingReports,
    /// The timeslot (τ)
    Timeslot,
    /// The authorization queue (φ)
    AuthorizationQueue,
    /// The privileged service indices (χ)
    Privileges,
    /// The disputes (ψ)
    Disputes,
    /// The activity statistics for the validators (π)
    Statistics,
    /// The accumulation queue (θ)
    Queue,
    /// The accumulation history (ξ)
    History,
}

/// A key in the service account
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceField {
    /// The service account data
    Data,
    /// The service account storage
    Storage,
    /// The service account preimage
    Preimage,
    /// The service account lookup table
    Lookup { length: u32 },
}

/// The kind of validator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidatorKind {
    /// The current validator (ι)
    Current,
    /// The previous validator (κ)
    Previous,
    /// The next validator (λ)
    Next,
}

impl StateKeyInfo for TrieKey {
    fn info(&self) -> StateKey {
        match *self {
            key::AUTHORIZATION_POOLS => StateKey::AuthorizationPools,
            key::AUTHORIZATION_QUEUE => StateKey::AuthorizationQueue,
            key::RECENT_BLOCKS => StateKey::RecentBlocks,
            key::SAFROLE => StateKey::Safrole,
            key::DISPUTES => StateKey::Disputes,
            key::ENTROPY => StateKey::Entropy,
            key::NEXT_VALIDATORS => StateKey::Validators {
                kind: ValidatorKind::Next,
            },
            key::CURRENT_VALIDATORS => StateKey::Validators {
                kind: ValidatorKind::Current,
            },
            key::PREVIOUS_VALIDATORS => StateKey::Validators {
                kind: ValidatorKind::Previous,
            },
            key::PENDING_REPORTS => StateKey::PendingReports,
            key::TIMESLOT => StateKey::Timeslot,
            key::PRIVILEGED_SERVICE => StateKey::Privileges,
            key::STATISTICS => StateKey::Statistics,
            key::ACCUMULATION_QUEUE => StateKey::Queue,
            key::ACCUMULATION_HISTORY => StateKey::History,
            key if key.starts_with(&[255]) => {
                let buf = [key[1], key[3], key[5], key[7]];
                StateKey::Account {
                    service: u32::from_le_bytes(buf),
                    field: ServiceField::Data,
                }
            }
            key if [key[1], key[3], key[5], key[7]] == key::ACCOUNT_STORAGE_PREFIX => {
                let buf = [key[0], key[2], key[4], key[6]];
                StateKey::Account {
                    service: u32::from_le_bytes(buf),
                    field: ServiceField::Storage,
                }
            }
            key if [key[1], key[3], key[5], key[7]] == key::ACCOUNT_PREIMAGE_PREFIX => {
                let buf = [key[0], key[2], key[4], key[6]];
                StateKey::Account {
                    service: u32::from_le_bytes(buf),
                    field: ServiceField::Preimage,
                }
            }
            key => {
                let buf = [key[0], key[2], key[4], key[6]];
                let service = u32::from_le_bytes(buf);
                let length = u32::from_le_bytes([key[1], key[3], key[5], key[7]]);
                StateKey::Account {
                    service,
                    field: ServiceField::Lookup { length },
                }
            }
        }
    }
}
