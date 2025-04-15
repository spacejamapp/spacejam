//! Parameters for the SpaceJam node.

use serde::{Deserialize, Serialize};

/// Parameters for spacejam
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Parameters {
    /// The parameters for version 1
    pub v1: V1,
}

/// Parameters for version 1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V1 {
    /// (B_S) The base deposit required to retain an account.
    pub deposit_per_account: u64,
    /// (B_I) The additional deposit required for storage item in an account.
    pub deposit_per_item: u64,
    /// (B_L)The additional deposit required for each byte of each storage item in an account and preimage of an account.
    pub deposit_per_byte: u64,
    /// (D) Minimum period in blocks between going from becoming Available to Zombie, and then again from Zombie to non-existent.
    pub min_turnaround_period: u32,
    /// (E) The epoch period, defined in number of slots.
    pub epoch_period: u32,
    /// (G_A) Maximum gas which may be used to accumulate a single work-report.
    pub max_accumulate_gas: u64,
    /// (G_I) Maximum gas which may be used to authorize a single work-package.
    pub max_is_authorized_gas: u64,
    /// (G_R) Maximum gas which may be used to refine a single work-report.
    pub max_refine_gas: u64,
    /// (G_T) Maximum gas which can be processed in a single block.
    pub block_gas_limit: u64,
    /// (H) The number of blocks which are kept in the recent block cache.
    pub recent_block_count: usize,
    /// (I) Maximum number of Work Items in a Work Package.
    pub max_work_items: u8,
    /// (J) Maximum number of dependencies (total of prerequisites and SR lookup entries).
    pub max_dependencies: usize,
    /// (K) Max tickets allowed to be embedded in each block extrinsic.
    pub max_tickets_per_block: usize,
    /// (L) Maximum age, in blocks, that the lookup anchor may be, taken from the regular anchor.
    pub max_lookup_anchor_age: u32,
    /// (N) The number of distinct tickets which may be created and submitted by each validator on each epoch.
    pub tickets_attempts_number: u8,
    /// (O) Number of items in the authorization window.
    pub auth_window: u8,
    /// (Q) Number of authorizations in a queue allocated to a core.
    pub auth_queue_len: u8,
    /// (R) The rotation period, defined in number of slots.
    pub rotation_period: u32,
    /// (T) Maximum number of extrinsics in a Work Package.
    pub max_extrinsics: u8,
    /// (U) The period in timeslots after which reported but unavailable work may be replaced.
    pub availability_timeout: u8,
    /// (V) Total number of validators.
    pub val_count: u16,
    /// (W_B)Maximum size of a Work Package together with all extrinsic data and imported segments.
    pub max_input: u32,
    /// (W_C) The maximum size of Refine/Accumulate code.
    pub max_refine_code_size: u32,
    /// (W_E) Number of octets in a erasure-coded piece.
    pub basic_piece_len: u16,
    /// (W_M) Maximum number of imports in a Work Package.
    pub max_imports: u16,
    /// (W_I) The maximum size of Is-Authorized code.
    pub max_is_authorized_code_size: usize,
    /// (W_R) The maximum amount of RAM which may be used by Refine/Accumulate code.
    pub max_refine_memory: usize,
    /// (W_U) The maximum amount of RAM which may be used by IsAuthorized code.
    pub max_is_authorized_memory: usize,
    /// (W_X) The maximum number of exports in a work package
    pub max_exports: usize,
    /// (C = V/3) The number of validators per core is always 3
    pub validators_per_core: u16,
    /// (W_G) The size of a segment
    pub segment_size: usize,
    /// (W_P) The number of erasure-coded pieces in a segment
    pub erasure_coded_pieces: u8,
}

impl Default for V1 {
    fn default() -> Self {
        Self {
            deposit_per_account: score::BALANCE_PER_SERVICE,
            deposit_per_item: score::BALANCE_PER_SERVICE,
            deposit_per_byte: score::BALANCE_PER_OCTET,
            min_turnaround_period: score::EXPUNGED_TIME,
            epoch_period: score::EPOCH_LENGTH,
            max_accumulate_gas: score::GAS_ACC,
            max_is_authorized_gas: score::GAS_IS_AUTHORIZED,
            max_refine_gas: score::GAS_REFINE,
            block_gas_limit: score::WORK_REPORT_GAS_LIMIT,
            recent_block_count: score::MAX_BLOCKS_HISTORY,
            max_work_items: score::MAX_WORK_ITEMS,
            max_dependencies: score::MAX_DEPENDENCY_COUNT,
            max_tickets_per_block: score::MAX_TICKETS_PER_EXTRINSIC,
            max_lookup_anchor_age: score::MAX_AGE_LOOKUP_ANCHOR,
            tickets_attempts_number: score::TICKET_ENTRIES_PER_VALIDATOR,
            auth_window: score::AUTH_WINDOW,
            auth_queue_len: score::AUTH_QUEUE_LEN,
            rotation_period: score::ROTATION_PERIOD,
            max_extrinsics: score::MAX_EXTRINSICS,
            availability_timeout: score::AVAILABILITY_TIMEOUT,
            val_count: score::VALIDATORS_COUNT,
            max_input: score::MAX_INPUT,
            max_refine_code_size: score::MAX_REFINE_CODE_SIZE,
            basic_piece_len: score::BASIC_PIECE_LEN,
            max_imports: score::MAX_IMPORTS_EXPORTS,
            max_is_authorized_code_size: score::MAX_IS_AUTHORIZED_CODE_SIZE,
            max_refine_memory: score::MAX_REFINE_MEMORY,
            max_is_authorized_memory: score::MAX_IS_AUTHORIZED_MEMORY,
            max_exports: score::MAX_EXPORTS,
            validators_per_core: score::VALIDATORS_COUNT / 3,
            segment_size: score::SEGMENT_SIZE,
            erasure_coded_pieces: score::ERASURE_CODED_PIECES,
        }
    }
}
