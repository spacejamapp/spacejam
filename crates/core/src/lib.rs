//! Core of SpaceJam

pub use {
    ::service::{self as svc, Array, vm, *},
    block::Block,
    codec,
    extrinsic::Extrinsic,
    state::{State, key::StorageKeyEncode},
};

#[cfg(feature = "blake2")]
pub use ::service::blake2b;

pub mod block;
pub mod extrinsic;
pub mod safrole;
pub mod service;
pub mod state;
pub mod statistic;

// crypto types

/// The type for a storage key
pub type TrieKey = [u8; 31];

/// The type for a bandersnatch public key
pub type BandersnatchPublic = [u8; 32];

/// The type for an ed25519 public key
pub type Ed25519Public = [u8; 32];

/// The type for a bls public key
pub type BlsPublic = [u8; 144];

/// The type for a bandersnatch vrf signature
pub type BandersnatchVrfSignature = [u8; 96];

/// The type for a bandersnatch ring commitment
pub type BandersnatchRingCommitment = [u8; 144];

/// The type for a bandersnatch ring vrf signature
pub type BandersnatchRingVrfSignature = [u8; 784];

/// The type for an ed25519 signature
pub type Ed25519Signature = [u8; 64];

// application specific core types

/// The type for an opaque hash
pub type OpaqueHash = [u8; 32];

/// The type for a timeslot
pub type TimeSlot = u32;

/// The type for a segment
pub type Segment = [u8; SEGMENT_SIZE];

/// The type for a validator index
pub type ValidatorIndex = u16;

/// The type for a core index
pub type CoreIndex = u16;

/// The type for a service id
pub type ServiceId = u32;

/// The type for a header hash
pub type HeaderHash = OpaqueHash;

/// The type for a state root
pub type StateRoot = OpaqueHash;

/// The type for a beefy root
pub type BeefyRoot = OpaqueHash;

/// The type for a work package hash
pub type WorkPackageHash = OpaqueHash;

/// The type for a work report hash
pub type WorkReportHash = OpaqueHash;

/// The type for an exports root
pub type ExportsRoot = OpaqueHash;

/// The type for an erasure root
pub type ErasureRoot = OpaqueHash;

/// The type for a gas
pub type Gas = u64;

/// The type for an entropy
pub type Entropy = OpaqueHash;

/// The type for an entropy buffer
pub type EntropyBuffer = [Entropy; 4];

/// The type for a validator metadata
pub type ValidatorMetadata = [u8; 128];
