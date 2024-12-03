//! Safrole vector tests
#![cfg(test)]

use codec::{Json, ResultJson};
use core::{
    misc::OpaqueHash,
    ticket::{TicketEnvelopeJson, TicketsExtrinsic},
};
use paste::paste;
use safrole::{Error, OutputData, OutputDataJson, State, StateJson};
use serde::{Deserialize, Serialize};
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
    #[json(ResultJson<OutputDataJson, Error>)]
    output: std::result::Result<OutputData, Error>,
    #[json(nested)]
    post_state: State,
}

impl Test {
    fn run(&self) -> anyhow::Result<()> {
        let mut state = self.pre_state.clone();
        let output = state.enact(
            self.input.slot,
            self.input.entropy,
            self.input.extrinsic.clone(),
        )?;

        assert_eq!(output, self.output);
        assert_eq!(state, self.post_state);
        Ok(())
    }
}

#[allow(unused_macros)]
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
