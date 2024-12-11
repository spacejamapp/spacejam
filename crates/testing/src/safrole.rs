//! Safrole vector tests
#![cfg(test)]

use paste::paste;
use safrole::{Error, Markers, MarkersJson, State, StateJson};
use score::{
    misc::OpaqueHash,
    ticket::{TicketEnvelopeJson, TicketsExtrinsic},
};
use serde::{Deserialize, Serialize};
use spacejson::{Json, ResultJson};
use std::{fs, path::PathBuf};

#[derive(Deserialize, Serialize, Json, Debug)]
struct Input {
    slot: u32,
    #[json(hex)]
    entropy: OpaqueHash,
    #[json(Vec<TicketEnvelopeJson>)]
    extrinsic: TicketsExtrinsic,
}

#[derive(Deserialize, Serialize, Json, Debug)]
pub struct Test {
    #[json(nested)]
    input: Input,
    #[json(nested)]
    pre_state: State,
    #[json(ResultJson<MarkersJson, Error>)]
    output: std::result::Result<Markers, Error>,
    #[json(nested)]
    post_state: State,
}

impl Test {
    fn run(&self) -> anyhow::Result<()> {
        crate::init_tracing();
        let mut state = self.pre_state.clone();
        let output = state.enact(
            self.input.slot,
            self.input.entropy,
            self.input.extrinsic.clone(),
        )?;

        assert_eq!(output, self.output, "Invalid output");
        assert_eq!(state.tau, self.post_state.tau, "Invalid time slot");
        assert_eq!(state.eta, self.post_state.eta, "Invalid entropy");
        assert_eq!(
            state.lambda, self.post_state.lambda,
            "Invalid previous epoch validators: lambda"
        );
        assert_eq!(
            state.kappa, self.post_state.kappa,
            "Invalid current epoch validators: kappa"
        );
        assert_eq!(
            state.iota, self.post_state.iota,
            "Validators to be drawn from next"
        );
        assert_eq!(
            state.gamma_k, self.post_state.gamma_k,
            "Invalid next epoch validators: gamma_k"
        );
        assert_eq!(
            state.gamma_z, self.post_state.gamma_z,
            "Invalid bandersnatch ring commitment: gamma_z"
        );
        assert_eq!(
            state.gamma_s, self.post_state.gamma_s,
            "Invalid sealing-key series: gamma_s"
        );
        assert_eq!(
            state.gamma_a, self.post_state.gamma_a,
            "Invalid sealing-key contest ticket accumulator: gamma_a"
        );
        Ok(())
    }
}

macro_rules! impl_safrole_tests {
    ($name:ident) => {
        paste! {
            #[test]
            fn [<$name:snake>]() -> anyhow::Result<()> {
                let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                root.extend(["jamtestvectors", "safrole", "tiny"]);
                root.push(stringify!($name).replace("_", "-"));
                root.set_extension("json");

                let json = fs::read_to_string(root)?;
                Test::from_json(&json)?.run()
            }
        }
    };
    ($($name:ident),*) => {
        $(impl_safrole_tests!($name);)*
    };
}

impl_safrole_tests! {
    enact_epoch_change_with_no_tickets_1,
    enact_epoch_change_with_no_tickets_2,
    enact_epoch_change_with_no_tickets_3,
    enact_epoch_change_with_no_tickets_4,
    enact_epoch_change_with_padding_1,
    publish_tickets_no_mark_1,
    publish_tickets_no_mark_2,
    publish_tickets_no_mark_3,
    publish_tickets_no_mark_4,
    publish_tickets_no_mark_5,
    publish_tickets_no_mark_6,
    publish_tickets_no_mark_7,
    publish_tickets_no_mark_8,
    publish_tickets_no_mark_9,
    publish_tickets_with_mark_1,
    publish_tickets_with_mark_2,
    publish_tickets_with_mark_3,
    publish_tickets_with_mark_4,
    publish_tickets_with_mark_5,
    skip_epoch_tail_1,
    skip_epochs_1
}
