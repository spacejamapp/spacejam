//! state transition traces

use score::{block::BlockJson, state::StateKeyLike, Block, OpaqueHash};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// State transition trace input
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct TestInput {
    /// The state
    #[json(nested)]
    pub pre_state: State,

    /// The block
    #[json(nested)]
    pub block: Block,
}

/// State transition trace output
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct TestOutput {
    /// The post-state
    #[json(nested)]
    pub post_state: State,
}

/// State transition trace state
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct State {
    /// The state root
    #[json(hex)]
    pub state_root: OpaqueHash,

    /// The key-values
    #[json(nested)]
    pub keyvals: Vec<KeyValue>,
}

/// State transition trace key-value
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct KeyValue {
    /// The key
    #[json(hex)]
    pub key: Vec<u8>,

    /// The value
    #[json(hex)]
    #[serde(with = "codec::vlen")]
    pub value: Vec<u8>,
}

#[test]
fn test_state_keys() {
    let registry = specjam::Registry::new("../../res/jam-test-vectors");
    let fallback = registry
        .trace(specjam::Trace::Fallback)
        .expect("failed to load fallback tests")
        .test("00000000")
        .expect("failed to load fallback test");

    let output = TestOutput::from_json(&fallback.output).expect("failed to parse output");
    let mut kvs = Vec::new();
    for keyval in output.post_state.keyvals {
        let key = keyval.key.as_state_key();
        let mut key31 = [0; 31];

        key31.copy_from_slice(&key[..31]);
        kvs.push((key31, keyval.value.clone()));
    }

    let state_root = crypto::merkle::trie31(&kvs);
    assert_eq!(
        state_root, output.post_state.state_root,
        "Calculated state root does not match expected value"
    );
}

mod fallback {
    include!(concat!(env!("OUT_DIR"), "/traces_fallback.rs"));
}

mod safrole {
    include!(concat!(env!("OUT_DIR"), "/traces_safrole.rs"));
}

mod reports_l0 {
    include!(concat!(env!("OUT_DIR"), "/traces_reports_l0.rs"));
}
