#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "std")]
pub(crate) use std::{collections::BTreeMap, string::String, vec::Vec};

#[cfg(not(feature = "std"))]
pub(crate) use alloc::{collections::BTreeMap, string::String, vec::Vec};

pub use params::Parameters;

pub mod api;
pub mod params;
pub mod service;
pub mod vm;

/// (B_I) The balance per item
pub const BALANCE_PER_ITEM: u64 = 10;

/// (B_L) The balance per octet
pub const BALANCE_PER_OCTET: u64 = 1;

/// (B_S) The balance per service
pub const BALANCE_PER_SERVICE: u64 = 100;

/// (C) The count of cores
pub const CORES_COUNT: usize = 2;

/// (G_A) The gas allocated to invoke a work report's Accumulation logic
pub const GAS_ACC: u64 = 10_000_000;

/// (G_T) The total gas allocated across for all accumulation
///
/// should be no smaller than G_A * C + ∑ privileges
pub const GAS_ALL_ACC: u64 = 20_000_000;

/// (W_G) The size of a segment
pub const SEGMENT_SIZE: usize = 4104;

/// (W_T) The size of a transfer memo
pub const TRANSFER_MEMO_SIZE: usize = 128;

/// The gas type
pub type Gas = u64;

/// The service id type
pub type ServiceId = u32;

/// The opaque hash type
pub type OpaqueHash = [u8; 32];

/// The time slot type
pub type TimeSlot = u32;

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

/// The type for an entropy buffer
pub type EntropyBuffer = [OpaqueHash; 4];

/// The type for a validator metadata
pub type ValidatorMetadata = [u8; 128];

/// The type for a bandersnatch public key
pub type BandersnatchPublic = [u8; 32];

/// The type for an ed25519 public key
pub type Ed25519Public = [u8; 32];

/// The type for a bls public key
pub type BlsPublic = [u8; 144];

#[cfg(feature = "blake2")]
/// Compute the BLAKE2b 256-bit hash of a given input.
pub fn blake2b(input: &[u8]) -> [u8; 32] {
    use blake2::{Blake2b, Digest, digest::consts::U32};

    let mut hasher = Blake2b::<U32>::new();
    hasher.update(input);
    hasher.finalize().into()
}
