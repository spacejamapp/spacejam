use core_derive::Json;

pub mod block;
pub mod dispute;
pub mod misc;
pub mod ticket;
pub mod work;

pub const VALIDATORS_COUNT: u16 = 6;
pub const CORES_COUNT: u16 = 2;
pub const EPOCH_LENGTH: u16 = 12;
pub const MAX_BLOCKS_HISTORY: u16 = 8;
pub const VALIDATORS_SUPER_MAJORITY: u16 = 5;
pub const AVAIL_BITFIELD_BYTES: u16 = 1;
