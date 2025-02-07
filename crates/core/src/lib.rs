//! Core of SpaceJam

pub use {block::Block, extrinsic::Extrinsic, state::State};

pub mod block;
pub mod extrinsic;
pub mod safrole;
pub mod service;
pub mod state;
pub mod statistic;
pub mod validator;
pub mod work;

/// The count of validators
pub const VALIDATORS_COUNT: u16 = 6;

/// The count of cores
pub const CORES_COUNT: usize = 2;

/// The length of an epoch
pub const EPOCH_LENGTH: u32 = 12;

/// The maximum number of blocks in the history
pub const MAX_BLOCKS_HISTORY: usize = 8;

/// The number of validators in a super majority
pub const VALIDATORS_SUPER_MAJORITY: u16 = 5;

/// The number of bytes in the avail bitfield
pub const AVAIL_BITFIELD_BYTES: usize = 1;

/// The maximum number of tickets per block
pub const MAX_TICKETS_PER_BLOCK: usize = 16;

/// The number of ticket entries per validator
pub const TICKET_ENTRIES_PER_VALIDATOR: u8 = 2;

/// The duration of a contest
pub const CONTEST_DURATION: u32 = 10;

/// The maximum size of a work report output
pub const MAX_WORK_REPORT_OUTPUT_SIZE: usize = 48 * 1024;

/// The rotation period of validator core assignments, in timeslots.
pub const ROTATION_PERIOD: u32 = 4;

/// The minimum gas for a service item.
pub const SERVICE_ITEM_MIN_GAS: u64 = 1000;

/// The maximum number of dependencies a work report can have.
pub const MAX_DEPENDENCY_COUNT: usize = 8;

/// FIXME: this number is extracted from the tests, I don't think it's correct.
pub const WORK_REPORT_GAS_LIMIT: u64 = 10_000_000;

/// The period in timeslots after which reported but unavailable work may be replaced.
pub const WORK_REPORT_TIMEOUT_PERIOD: u32 = 5;

/// The common era of the jam (4.4)
///
/// The beginning of the jam common era, 1200 UTC on Jan 1, 2025
///
/// (1_735_689_600) after the unix epoch
pub const JAM_COMMON_ERA_AFTER_UNIX_EPOCH: u32 = 1_735_689_600;

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
