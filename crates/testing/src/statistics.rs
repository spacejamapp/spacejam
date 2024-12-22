#![cfg(test)]

use score::{
    block::{Extrinsic, ExtrinsicJson},
    Ed25519Public, TimeSlot, ValidatorIndex,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;
use statistic::{State, StateJson, Stats};

#[derive(Debug, PartialEq, Eq, Json, Serialize, Deserialize)]
pub struct Input {
    slot: TimeSlot,
    author_index: ValidatorIndex,
    #[json(nested)]
    extrinsic: Extrinsic,
    #[json(Vec<String>)]
    reporters: Vec<Ed25519Public>,
}

#[derive(Json, Serialize, Deserialize, Debug)]
struct Test {
    #[json(nested)]
    pre_state: State,
    #[json(nested)]
    input: Input,
    output: (),
    #[json(nested)]
    post_state: State,
}

impl Test {
    fn run(self) {
        let stats = Stats::from(self.pre_state);
        let stats = stats.update(
            self.input.slot,
            self.input.author_index,
            self.input.extrinsic,
            self.input.reporters,
        );

        assert_eq!(
            stats.next_state.pi.current, self.post_state.pi.current,
            "Invalid current pi"
        );
        assert_eq!(
            stats.next_state.pi.last, self.post_state.pi.last,
            "Invalid last pi"
        );
        assert_eq!(stats.state.tau, self.post_state.tau, "Invalid tau");
        assert_eq!(
            stats.state.kappa_prime, self.post_state.kappa_prime,
            "Invalid kappa_prime"
        );
    }
}

crate::impl_statistics_tests! {
    stats_with_empty_extrinsic_1,
    stats_with_epoch_change_1,
    stats_with_some_extrinsic_1
}
