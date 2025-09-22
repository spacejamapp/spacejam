//! Core assignments

use score::{
    CORES_COUNT, CoreIndex, EPOCH_LENGTH, Entropy, EntropyBuffer, ROTATION_PERIOD, TimeSlot,
    VALIDATORS_COUNT,
};

/// Core assignments based on the timeslot and entropy
pub fn core(timeslot: u32, eta: [u8; 32]) -> [Vec<u16>; score::CORES_COUNT] {
    let shuffled = crypto::shuffle::eq331(
        &(0..score::VALIDATORS_COUNT as u32)
            .map(|i| (i * score::CORES_COUNT as u32) / score::VALIDATORS_COUNT as u32)
            .collect::<Vec<_>>(),
        eta,
    );

    // Calculate rotation offset based on timeslot (P function)
    let rotation = (timeslot % score::EPOCH_LENGTH) / score::ROTATION_PERIOD as u32;
    let rotated: Vec<u32> = shuffled
        .iter()
        .map(|&core_idx| (core_idx + rotation) % score::CORES_COUNT as u32)
        .collect();

    // Group validators by their assigned cores
    let mut assignments: [Vec<u16>; score::CORES_COUNT] = Default::default();
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
