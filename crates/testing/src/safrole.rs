//! Safrole vector tests
#![cfg(test)]

use codec::Json;
use core::{misc::OpaqueHash, ticket::TicketsExtrinsic};
use paste::paste;
use safrole::{Error, OutputData, OutputDataJson, State, StateJson};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Deserialize, Serialize, Json, Debug)]
struct Input {
    slot: u32,
    #[json(hex)]
    entropy: OpaqueHash,
    #[json(nested)]
    extrinsic: TicketsExtrinsic,
}

#[derive(Deserialize, Serialize, Json, Debug)]
pub struct Test {
    #[json(nested)]
    input: Input,
    #[json(nested)]
    pre_state: State,
    #[json(nested)]
    output: std::result::Result<OutputData, Error>,
    #[json(nested)]
    post_state: State,
}

#[allow(unused_macros)]
macro_rules! impl_safrole_tests {
    ($name:tt) => {
        paste! {
            #[test]
            fn [<test_ $name:snake>]() -> anyhow::Result<()> {
                let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                root.extend(["jamtestvectors", "safrole", "tiny"]);
                root.push(stringify!($name).replace("_", "-"));
                root.set_extension("json");

                let json = fs::read_to_string(root)?;
                let _: Test = serde_json::from_str(&json)?;
                Ok(())
            }
        }
    };
}

impl_safrole_tests! {
    enact_epoch_change_with_no_tickets_1
}
