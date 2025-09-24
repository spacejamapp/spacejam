//! Tiny parameters for testing

use crate::Parameters;

impl Parameters {
    /// Tiny parameters for testing
    pub const fn tiny() -> Self {
        Self {
            deposit_per_item: 10,
            deposit_per_byte: 1,
            deposit_per_account: 100,
            validators_per_core: 2,
            min_turnaround_period: 32,
            epoch_period: 12,
            max_accumulate_gas: 10000000,
            max_is_authorized_gas: 50000000,
            max_refine_gas: 1000000000,
            block_gas_limit: 20000000,
            recent_block_count: 8,
            max_work_items: 16,
            max_dependencies: 8,
            max_tickets_per_extrinsic: 3,
            max_lookup_anchor_age: 24,
            ticket_entries_per_validator: 3,
            auth_window: 8,
            slot_period: 6,
            auth_queue_len: 80,
            rotation_period: 4,
            max_extrinsics: 128,
            availability_timeout: 5,
            val_count: 6,
            max_is_authorized_code_size: 64000,
            max_input: 13794305,
            max_refine_code_size: 4000000,
            basic_piece_len: 4,
            max_imports: 3072,
            erasure_coded_pieces: 1026,
            max_refine_memory: 49152,
            transfer_memo_size: 128,
            max_exports: 3072,
            ticket_submission_period: 10,
        }
    }
}
