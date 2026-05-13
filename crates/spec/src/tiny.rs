//! Tiny chain-spec constants

/// (C) The count of cores
pub const CORES_COUNT: usize = 2;

/// (D) The period in timeslots after which an unreferenced preimage may be expunged.
pub const EXPUNGED_TIME: u32 = 32;

/// (E) The length of an epoch
pub const EPOCH_LENGTH: u32 = 12;

/// (G_R) The gas allocated to invoke a work report's Refine logic
pub const GAS_REFINE: u64 = 5_000_000_000;

/// (G_T) The total gas allocated across for all accumulation
pub const GAS_ALL_ACC: u64 = 20_000_000;

/// (K) The maximum number of tickets which may be submitted in a single extrinsic.
pub const MAX_TICKETS_PER_EXTRINSIC: u16 = 16;

/// (L) The maximum age of a lookup anchor (in timeslots).
pub const MAX_AGE_LOOKUP_ANCHOR: u32 = 24;

/// (N) The number of ticket entries per validator
pub const TICKET_ENTRIES_PER_VALIDATOR: u16 = 2;

/// (R) The rotation period of validator core assignments, in timeslots.
pub const ROTATION_PERIOD: u16 = 4;

/// (V) The count of validators
pub const VALIDATORS_COUNT: u16 = 6;

/// (W_E) The number of octets in an erasure-coded piece.
pub const BASIC_PIECE_LEN: u32 = 4;

/// (W_P) The number of erasure-coded pieces in a segment
pub const ERASURE_CODED_PIECES: u32 = 1026;

/// (Y) The number of slots into an epoch at which ticket-submission ends.
pub const TICKET_SUBMISSION_PERIOD: u32 = 10;

/// The number of validators in a super majority (ceil(V * 2/3 + 1))
pub const VALIDATORS_SUPER_MAJORITY: u16 = 5;

/// The number of bytes in the avail bitfield (floor((C + 7) / 8))
pub const AVAIL_BITFIELD_BYTES: usize = 1;
