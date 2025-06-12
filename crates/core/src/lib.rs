//! Core of SpaceJam
//!
//! TODO: remove crypto as dependency

pub use {
    block::Block,
    extrinsic::Extrinsic,
    state::{key::StorageKeyEncode, State},
};

pub mod block;
pub mod extrinsic;
pub mod safrole;
pub mod service;
pub mod state;
pub mod statistic;
pub mod vm;

/// (B_I) The balance per item
pub const BALANCE_PER_ITEM: u64 = 10;

/// (B_L) The balance per octet
pub const BALANCE_PER_OCTET: u64 = 1;

/// (B_S) The balance per service
pub const BALANCE_PER_SERVICE: u64 = 100;

/// (C) The count of cores
pub const CORES_COUNT: usize = 2;

/// (D) The period in timeslots after which an unreferenced preimage may be expunged.
pub const EXPUNGED_TIME: u32 = 192_000;

/// (E) The length of an epoch
pub const EPOCH_LENGTH: u32 = 12;

/// (G_A) The gas allocated to invoke a work report's Accumulation logic
pub const GAS_ACC: u64 = 10_000_000;

/// (G_I) The gas allocated to invoke a work report's IsAuthorized logic
pub const GAS_IS_AUTHORIZED: u64 = 50_000_000;

/// (G_R) The gas allocated to invoke a work report's Refine logic
pub const GAS_REFINE: u64 = 5_000_000_000;

/// (G_T) The total gas allocated across for all accumulation
///
/// should be no smaller than G_A * C + ∑ privileges
pub const GAS_ALL_ACC: u64 = 3_500_000_000;

/// (H) The maximum number of blocks in the history
pub const MAX_BLOCKS_HISTORY: usize = 8;

/// (I) The maximum number of work items in a work package
pub const MAX_WORK_ITEMS: u8 = 16;

/// (J) The maximum number of dependencies a work report can have.
pub const MAX_DEPENDENCY_COUNT: usize = 8;

/// (K) The maximum number of tickets which may be submitted in a single extrinsic.
pub const MAX_TICKETS_PER_EXTRINSIC: usize = 16;

/// (L) The maximum age of a lookup anchor (24 hrs)
pub const MAX_AGE_LOOKUP_ANCHOR: u32 = 24 * 60 * 60 / SLOT_PERIOD;

/// (N) The number of ticket entries per validator
pub const TICKET_ENTRIES_PER_VALIDATOR: u8 = 2;

/// (N_Q) The number of items in the authorization queue
pub const QUEUE_ITEMS: u64 = 80;

/// (O) The maximum number of items in the authorizations pool
pub const AUTH_WINDOW: u8 = 8;

/// (Q) The number of items in the authorizations queue
pub const AUTH_QUEUE_LEN: u8 = 60;

/// (R) The rotation period of validator core assignments, in timeslots.
pub const ROTATION_PERIOD: u32 = 4;

/// (T) The maximum number of extrinsics in a work package
pub const MAX_EXTRINSICS: u8 = 128;

/// (U) The period in timeslots after which reported but unavailable work may be replaced.
pub const AVAILABILITY_TIMEOUT: u8 = 5;

/// (V) The count of validators
pub const VALIDATORS_COUNT: u16 = 6;

/// (W_B) The maximum size of a work package together with all extrinsic data and imported segments.
pub const MAX_INPUT: u32 = 12 * (1 << 20);

/// (W_C) The maximum size of Refine/Accumulate code.
pub const MAX_REFINE_CODE_SIZE: u32 = 4_000_000;

/// (W_E) The number of octets in a erasure-coded piece.
pub const BASIC_PIECE_LEN: u16 = 684;

/// (W_G) The size of a segment in octets
pub const SEGMENT_SIZE: usize = 4104;

/// (W_I) The maximum is authorized code size
pub const MAX_IS_AUTHORIZED_CODE_SIZE: usize = 0;

/// (W_R) The maximum amount of RAM which may be used by Refine/Accumulate code.
pub const MAX_REFINE_MEMORY: usize = 0;

/// (W_U) The maximum amount of RAM which may be used by IsAuthorized code.
pub const MAX_IS_AUTHORIZED_MEMORY: usize = 0;

/// (W_M) The maximum number of imports and exports in a work package
pub const MAX_IMPORTS_EXPORTS: u16 = 3072;

/// (W_P) The number of erasure-coded pieces in a segment
pub const ERASURE_CODED_PIECES: u8 = 6;

/// (W_T) The size of the transfer memo
pub const TRANSFER_MEMO_SIZE: u32 = 128;

/// (W_X) The maximum number of exports in a work package
pub const MAX_EXPORTS: usize = 0;

/// (Y) The number of slots into an epoch at which ticket-submission ends.
pub const TICKET_SUBMISSION_PERIOD: u32 = 10;

/// The size of the program init data
///
/// 2^24 = 16_777_216 bytes
pub const PROGRAM_INIT_DATA_SIZE: usize = 1 << 24;

/// (Z_P) The size of a page in octets (2^12)
pub const PAGE_SIZE: usize = 1 << 12;

/// (Z_Z) The size of a zone in octets (2^16)
pub const ZONE_SIZE: usize = 1 << 16;

/// (Z_I) The size of the init data in octets (2^24)
pub const PVM_INIT_DATA_SIZE: usize = 1 << 24;

/// The number of validators in a super majority
pub const VALIDATORS_SUPER_MAJORITY: u16 = 5;

/// The number of bytes in the avail bitfield
pub const AVAIL_BITFIELD_BYTES: usize = 1;

/// (W_R) The maximum size of a work report output
pub const MAX_WORK_REPORT_OUTPUT_SIZE: usize = 48 * 1024;

/// The minimum gas for a service item.
pub const SERVICE_ITEM_MIN_GAS: u64 = 1000;

/// FIXME: this number is extracted from the tests, I don't think it's correct.
pub const WORK_REPORT_GAS_LIMIT: u64 = 10_000_000;

/// The period in timeslots after which reported but unavailable work may be replaced.
pub const WORK_REPORT_TIMEOUT_PERIOD: u32 = 5;

/// The common era of the jam (4.4)
///
/// The beginning of the jam common era, 1200 UTC on Jan 1, 2025
///
/// (1_735_732_800) after the unix epoch
pub const JAM_COMMON_ERA_AFTER_UNIX_EPOCH: u32 = 1_735_732_800;

/// The period of a timeslot in seconds
pub const SLOT_PERIOD: u32 = 6;

/// The length of pages, p = 2^32 / 2^12
pub const PAGE_LENGTH: usize = 1 << 20;

/// The size of the PVM memory
pub const PVM_MEMORY_SIZE: usize = 1 << 32;

/// The size of the PVM zone
pub const PVM_ZONE_SIZE: usize = 1 << 16;

/// The maximum size of the authorization pool
pub const AUTH_POOL_MAX_SIZE: usize = 8;

// Singing Contexts

/// The signing context for the jam available key
pub const JAM_AVAILABLE: [u8; 13] = *b"jam_available";

/// The signing context for the jam valid key
pub const JAM_VALID: [u8; 9] = *b"jam_valid";

/// The signing context for the jam invalid key
pub const JAM_INVALID: [u8; 11] = *b"jam_invalid";

/// The signing context for the jam guarantee key
pub const JAM_GUARANTEE: [u8; 13] = *b"jam_guarantee";

/// The signing context for the jam entropy key
pub const JAM_ENTROPY: [u8; 11] = *b"jam_entropy";

/// The signing context for the jam ticket seal key
pub const JAM_TICKET_SEAL: [u8; 15] = *b"jam_ticket_seal";

/// The signing context for the jam fallback seal key
pub const JAM_FALLBACK_SEAL: [u8; 17] = *b"jam_fallback_seal";

// crypto types

/// The type for a storage key
pub type StorageKey = [u8; 31];

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
