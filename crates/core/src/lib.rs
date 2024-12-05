pub mod block;
pub mod dispute;
pub mod misc;
pub mod ticket;
pub mod work;

pub const VALIDATORS_COUNT: u16 = 6;
pub const CORES_COUNT: u16 = 2;
pub const EPOCH_LENGTH: u32 = 12;
pub const MAX_BLOCKS_HISTORY: u16 = 8;
pub const VALIDATORS_SUPER_MAJORITY: u16 = 5;
pub const AVAIL_BITFIELD_BYTES: u16 = 1;
pub const SUBMISSION_PERIOD: u32 = 6;
pub const MAX_TICKETS_PER_BLOCK: usize = 16;
pub const TICKET_ENTRIES_PER_VALIDATOR: u8 = 2;
pub const CONTEST_DURATION: u32 = 10;
