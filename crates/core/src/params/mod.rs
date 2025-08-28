//! Chain parameters

use serde::{Deserialize, Serialize};
use std::sync::{LazyLock, RwLock, RwLockReadGuard};

mod tiny;

/// Parameters for the chain
static PARAMS: LazyLock<RwLock<Parameters>> = LazyLock::new(|| RwLock::new(Parameters::tiny()));

/// Get the parameters
pub fn get() -> RwLockReadGuard<'static, Parameters> {
    PARAMS.read().expect("Failed to read parameters")
}

/// Set new parameters
pub fn set(params: Parameters) {
    *PARAMS.write().expect("Failed to write parameters") = params;
}

/// Parameters for version 1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameters {
    /// (B_I) The additional deposit required for storage item in an account.
    pub deposit_per_item: u64,
    /// (B_L)The additional deposit required for each byte of each storage item in an account and preimage of an account.
    pub deposit_per_byte: u64,
    /// (B_S) The base deposit required to retain an account.
    pub deposit_per_account: u64,
    /// (C = V/3) The number of validators per core is always 3
    pub validators_per_core: u16,
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
    pub recent_block_count: u16,
    /// (I) Maximum number of Work Items in a Work Package.
    pub max_work_items: u16,
    /// (J) Maximum number of dependencies (total of prerequisites and SR lookup entries).
    pub max_dependencies: u16,
    /// (K) The maximum number of tickets which may be submitted in a single extrinsic.
    pub max_tickets_per_extrinsic: u16,
    /// (L) Maximum age, in blocks, that the lookup anchor may be, taken from the regular anchor.
    pub max_lookup_anchor_age: u32,
    /// (N) The number of ticket entries per validator
    pub ticket_entries_per_validator: u16,
    /// (O) Number of items in the authorization window.
    pub auth_window: u16,
    /// (P) the slot period
    pub slot_period: u16,
    /// (Q) Number of authorizations in a queue allocated to a core.
    pub auth_queue_len: u16,
    /// (R) The rotation period, defined in number of slots.
    pub rotation_period: u16,
    /// (T) Maximum number of extrinsics in a Work Package.
    pub max_extrinsics: u16,
    /// (U) The period in timeslots after which reported but unavailable work may be replaced.
    pub availability_timeout: u16,
    /// (V) Total number of validators.
    pub val_count: u16,
    /// (W_A) the max size of is-authorized code
    pub max_is_authorized_code_size: u32,
    /// (W_B) Maximum size of a Work Package together with all extrinsic data and imported segments.
    pub max_input: u32,
    /// (W_C) The maximum size of Refine/Accumulate code.
    pub max_refine_code_size: u32,
    /// (W_E) Number of octets in a erasure-coded piece.
    pub basic_piece_len: u32,
    /// (W_M) Maximum number of imports in a Work Package.
    pub max_imports: u32,
    /// (W_P) The number of erasure-coded pieces in a segment
    pub erasure_coded_pieces: u32,
    /// (W_R) The maximum amount of RAM which may be used by Refine/Accumulate code.
    pub max_refine_memory: u32,
    /// (W_T) the size of the transfer memo
    pub transfer_memo_size: u32,
    /// (W_X) The maximum number of exports in a work package
    pub max_exports: u32,
    /// (Y) The ticket submission period
    pub ticket_submission_period: u32,
}

impl Default for Parameters {
    fn default() -> Self {
        Self::tiny()
    }
}

/// Guarantor assignments for the chain
#[cfg(feature = "shuffle")]
pub mod assignments {
    use crate::{
        CoreIndex, Entropy, EntropyBuffer, TimeSlot, CORES_COUNT, EPOCH_LENGTH, ROTATION_PERIOD,
        VALIDATORS_COUNT,
    };

    /// Core assignments based on the timeslot and entropy
    pub fn core(timeslot: u32, eta: [u8; 32]) -> [Vec<u16>; crate::CORES_COUNT] {
        let shuffled = crypto::shuffle::eq331(
            &(0..crate::VALIDATORS_COUNT as u32)
                .map(|i| (i * crate::CORES_COUNT as u32) / crate::VALIDATORS_COUNT as u32)
                .collect::<Vec<_>>(),
            eta,
        );

        // Calculate rotation offset based on timeslot (P function)
        let rotation = (timeslot % crate::EPOCH_LENGTH) / crate::ROTATION_PERIOD as u32;
        let rotated: Vec<u32> = shuffled
            .iter()
            .map(|&core_idx| (core_idx + rotation) % crate::CORES_COUNT as u32)
            .collect();

        // Group validators by their assigned cores
        let mut assignments: [Vec<u16>; crate::CORES_COUNT] = Default::default();
        for (validator_idx, &core_idx) in rotated.iter().enumerate() {
            assignments[core_idx as usize].push(validator_idx as u16);
        }

        assignments
    }

    /// (M*) Get the guarantor assignments for the previous rotation period.
    ///
    /// This is needed for validating guarantees from the previous rotation.
    pub fn last(entropy: EntropyBuffer, timeslot: TimeSlot) -> Vec<CoreIndex> {
        let previous_timeslot = timeslot.saturating_sub(ROTATION_PERIOD as u32);

        // Check if we're in the same epoch
        let current_epoch = timeslot / EPOCH_LENGTH;
        let previous_epoch = previous_timeslot / EPOCH_LENGTH;
        let entropy = if current_epoch == previous_epoch {
            entropy[2]
        } else {
            entropy[3]
        };

        self::permute(entropy, previous_timeslot)
    }

    /// (M) Get the guarantor assignments for the current rotation period.
    pub fn current(entropy: EntropyBuffer, timeslot: TimeSlot) -> Vec<CoreIndex> {
        self::permute(entropy[2], timeslot)
    }

    /// Permute function P(e, t) for guarantor assignments.
    ///
    /// Returns the core assignments for all validators based on entropy and time.
    fn permute(entropy: Entropy, timeslot: TimeSlot) -> Vec<CoreIndex> {
        let initial_assignments: Vec<u32> = (0..VALIDATORS_COUNT as u32)
            .map(|i| (CORES_COUNT as u32 * i) / VALIDATORS_COUNT as u32)
            .collect();
        let shuffled = crypto::shuffle::eq331(&initial_assignments, entropy);

        // Apply rotation and convert to CoreIndex
        let rotation_offset = (timeslot % EPOCH_LENGTH) / ROTATION_PERIOD as u32;
        self::rotate(
            shuffled.into_iter().map(|x| x as CoreIndex).collect(),
            rotation_offset,
        )
    }

    /// Rotation function R for guarantor assignments.
    ///
    /// Rotates core assignments by n positions.
    fn rotate(assignments: Vec<CoreIndex>, n: u32) -> Vec<CoreIndex> {
        assignments
            .iter()
            .map(|&x| ((x as u32 + n) % CORES_COUNT as u32) as CoreIndex)
            .collect()
    }
}
