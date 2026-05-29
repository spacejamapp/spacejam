//! Statistics tests

use score::{
    EPOCH_LENGTH, TimeSlot, ValidatorIndex,
    extrinsic::Extrinsic,
    safrole::{ValidatorIter, ValidatorsData},
    statistic::{Statistics, ValidatorStats},
};
use serde::{Deserialize, Serialize};

include!(concat!(env!("OUT_DIR"), "/statistics.rs"));

/// The statistics STF `State` raw layout:
/// `(vals-curr-stats, vals-last-stats, slot, curr-validators)`.
type RawState = (ValidatorStats, ValidatorStats, TimeSlot, ValidatorsData);

/// Run the statistics test
pub fn run(test: &specjam::Test) -> anyhow::Result<()> {
    // The statistics STF `Output` is ASN.1 `NULL` (zero raw bytes).
    let (input, pre, (), post) =
        codec::decode::<(Input, RawState, (), RawState)>(test.input.expect_bin()?)?;
    let new_epoch = input.slot / EPOCH_LENGTH > pre.2 / EPOCH_LENGTH;
    let validators = pre.3;
    let mut stats = Statistics {
        vals_current: pre.0,
        vals_last: pre.1,
        ..Default::default()
    };
    stats.update(new_epoch, input.author_index, &input.extrinsic)?;

    // Per-validator guarantee credit comes from the reporters (the validators
    // that signed each guarantee), mirroring the executor's `merge_reporters`.
    let reporters: Vec<_> = input
        .extrinsic
        .guarantees
        .iter()
        .flat_map(|g| {
            g.signatures
                .iter()
                .map(|s| validators[s.validator_index as usize].ed25519)
        })
        .collect();
    stats.merge_reporters(&reporters, &validators.ed25519())?;

    assert_eq!(stats.vals_current, post.0);
    assert_eq!(stats.vals_last, post.1);
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Input {
    pub slot: TimeSlot,
    pub author_index: ValidatorIndex,
    pub extrinsic: Extrinsic,
}
