//! Chain-spec constants and parameters for SpaceJam
#![no_std]
#![warn(missing_docs)]

#[cfg(not(any(feature = "tiny", feature = "full")))]
compile_error!("enable one of `tiny` or `full` features");

mod param;
#[cfg(all(feature = "tiny", not(feature = "full")))]
mod tiny;
#[cfg(feature = "full")]
mod full;

pub use param::Parameters;

#[cfg(all(feature = "tiny", not(feature = "full")))]
pub use tiny::*;
#[cfg(feature = "full")]
pub use full::*;

// Universal constants

/// (B_I) The balance per item
pub const BALANCE_PER_ITEM: u64 = 10;

/// (B_L) The balance per octet
pub const BALANCE_PER_OCTET: u64 = 1;

/// (B_S) The balance per service
pub const BALANCE_PER_SERVICE: u64 = 100;

/// (G_A) The gas allocated to invoke a work report's Accumulation logic
pub const GAS_ACC: u64 = 10_000_000;

/// (G_I) The gas allocated to invoke a work report's IsAuthorized logic
pub const GAS_IS_AUTHORIZED: u64 = 50_000_000;

/// (H) The maximum number of blocks in the history
pub const MAX_BLOCKS_HISTORY: u16 = 8;

/// (I) The maximum number of work items in a work package
pub const MAX_WORK_ITEMS: u16 = 16;

/// (J) The maximum number of dependencies a work report can have.
pub const MAX_DEPENDENCY_COUNT: u16 = 8;

/// (O) The maximum number of items in the authorizations pool
pub const AUTH_WINDOW: u16 = 8;

/// (P) The slot period
pub const SLOT_PERIOD: u16 = 6;

/// (Q) The number of items in the authorizations queue
pub const QUEUE_ITEMS: u16 = 80;

/// (S) The minimum service id
pub const MINIMUM_SERVICE_ID: u32 = 65536;

/// (T) The maximum number of extrinsics in a work package
pub const MAX_EXTRINSICS: u16 = 128;

/// (U) The period in timeslots after which reported but unavailable work may be replaced.
pub const AVAILABILITY_TIMEOUT: u16 = 5;

/// (W_A) The maximum size of is-authorized code in octets
pub const MAX_IS_AUTHORIZED_CODE_SIZE: u32 = 64_000;

/// (W_B) The maximum size of a work package together with all extrinsic data and imported segments.
pub const MAX_INPUT: u32 = 12 * (1 << 20);

/// (W_C) The maximum size of Refine/Accumulate code.
pub const MAX_REFINE_CODE_SIZE: u32 = 4_000_000;

/// (W_G) The size of a segment in octets
pub const SEGMENT_SIZE: usize = 4104;

/// (W_R) The maximum amount of RAM which may be used by Refine/Accumulate code.
pub const MAX_REFINE_MEMORY: u32 = 0;

/// (W_U) The maximum amount of RAM which may be used by IsAuthorized code.
pub const MAX_IS_AUTHORIZED_MEMORY: usize = 0;

/// (W_M) The maximum number of imports and exports in a work package
pub const MAX_IMPORTS: u32 = 3072;

/// (W_T) The size of the transfer memo
pub const TRANSFER_MEMO_SIZE: usize = 128;

/// (W_X) The maximum number of exports in a work package
pub const MAX_EXPORTS: u32 = 3072;

/// The size of the program init data (2^24 bytes)
pub const PROGRAM_INIT_DATA_SIZE: usize = 1 << 24;

/// (Z_P) The size of a page in octets (2^12)
pub const PAGE_SIZE: usize = 1 << 12;

/// (Z_Z) The size of a zone in octets (2^16)
pub const ZONE_SIZE: usize = 1 << 16;

/// (Z_I) The size of the init data in octets (2^24)
pub const PVM_INIT_DATA_SIZE: usize = 1 << 24;

/// The maximum size of a work report output
pub const MAX_WORK_REPORT_OUTPUT_SIZE: usize = 48 * 1024;

/// The minimum gas for a service item.
pub const SERVICE_ITEM_MIN_GAS: u64 = 1000;

/// The work report gas limit.
pub const WORK_REPORT_GAS_LIMIT: u64 = 10_000_000;

/// The period in timeslots after which reported but unavailable work may be replaced.
pub const WORK_REPORT_TIMEOUT_PERIOD: u32 = 5;

/// The beginning of the jam common era, 1200 UTC on Jan 1, 2025 (1_735_732_800 after the unix epoch).
pub const JAM_COMMON_ERA_AFTER_UNIX_EPOCH: u32 = 1_735_732_800;

/// The length of pages, p = 2^32 / 2^12
pub const PAGE_LENGTH: usize = 1 << 20;

/// The size of the PVM memory
pub const PVM_MEMORY_SIZE: usize = 1 << 32;

/// The size of the PVM zone
pub const PVM_ZONE_SIZE: usize = 1 << 16;

/// The maximum size of the authorization pool
pub const AUTH_POOL_MAX_SIZE: usize = 8;

/// (Q) The size of the authorization queue per core
pub const AUTH_QUEUE_SIZE: usize = 80;

/// The number of guarantors per core
pub const GUARANTORS_PER_CORE: usize = VALIDATORS_COUNT as usize / CORES_COUNT;

/// The salt for the check function
pub const CHECK_SALT: u32 = ((1u64 << 32) - (1u64 << 8)) as u32 - MINIMUM_SERVICE_ID;

// Signing contexts

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
