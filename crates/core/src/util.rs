//! core utilities

#[cfg(feature = "shuffle")]
/// Core assignments based on the timeslot and entropy
pub fn assignments(timeslot: u32, eta: [u8; 32]) -> [Vec<u16>; crate::CORES_COUNT] {
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
