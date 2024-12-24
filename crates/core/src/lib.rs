//! Core of SpaceJam

pub use config::Config;

pub mod block;
pub mod config;
pub mod extrinsic;
pub mod safrole;
pub mod service;
pub mod state;
pub mod statistic;
pub mod validator;
pub mod work;

pub const VALIDATORS_COUNT: u16 = 6;
pub const CORES_COUNT: usize = 2;
pub const EPOCH_LENGTH: u32 = 12;
pub const MAX_BLOCKS_HISTORY: usize = 8;
pub const VALIDATORS_SUPER_MAJORITY: u16 = 5;
pub const AVAIL_BITFIELD_BYTES: usize = 1;
pub const MAX_TICKETS_PER_BLOCK: usize = 16;
pub const TICKET_ENTRIES_PER_VALIDATOR: u8 = 2;
pub const CONTEST_DURATION: u32 = 10;
pub const MAX_WORK_REPORT_OUTPUT_SIZE: usize = 48 * 1024;

/// The rotation period of validator core assignments, in timeslots.
pub const ROTATION_PERIOD: u32 = 4;

/// The minimum gas for a service item.
pub const SERVICE_ITEM_MIN_GAS: u64 = 1000;

/// The maximum number of dependencies a work report can have.
pub const MAX_DEPENDENCY_COUNT: usize = 8;

/// FIXME: this number is extracted from the tests, I don't think it's correct.
pub const WORK_REPORT_GAS_LIMIT: u64 = 10_000_000;

// Singing Contexts
pub const JAM_AVAILABLE: [u8; 13] = *b"jam_available";
pub const JAM_VALID: [u8; 9] = *b"jam_valid";
pub const JAM_INVALID: [u8; 11] = *b"jam_invalid";
pub const JAM_GUARANTEE: [u8; 13] = *b"jam_guarantee";

// crypto types
pub type BandersnatchPublic = [u8; 32];
pub type Ed25519Public = [u8; 32];
pub type BlsPublic = [u8; 144];
pub type BandersnatchVrfSignature = [u8; 96];
pub type BandersnatchRingCommitment = [u8; 144];
pub type BandersnatchRingVrfSignature = [u8; 784];
pub type Ed25519Signature = [u8; 64];

// application specific core types
pub type OpaqueHash = [u8; 32];
pub type TimeSlot = u32;
pub type ValidatorIndex = u16;
pub type CoreIndex = u16;
pub type ServiceId = u32;

pub type HeaderHash = OpaqueHash;
pub type StateRoot = OpaqueHash;
pub type BeefyRoot = OpaqueHash;
pub type WorkPackageHash = OpaqueHash;
pub type WorkReportHash = OpaqueHash;
pub type ExportsRoot = OpaqueHash;
pub type ErasureRoot = OpaqueHash;

pub type Gas = u64;
pub type Entropy = OpaqueHash;
pub type EntropyBuffer = [Entropy; 4];
pub type ValidatorMetadata = [u8; 128];
