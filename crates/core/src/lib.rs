pub mod block;
pub mod dispute;
pub mod extrinsic;
pub mod misc;
pub mod stats;
pub mod ticket;
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

// Singing Contexts

pub const JAM_AVAILABLE: [u8; 13] = *b"jam_available";
pub const JAM_VALID: [u8; 9] = *b"jam_valid";
pub const JAM_INVALID: [u8; 11] = *b"jam_invalid";
pub const JAM_GUARANTEE: [u8; 13] = *b"jam_guarantee";
