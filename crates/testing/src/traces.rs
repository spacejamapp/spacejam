//! state transition traces

use crypto::merkle;
use score::{
    block::BlockJson,
    state::{StateKeyInfo, StateKeyLike},
    Block, OpaqueHash,
};
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

    // let input = TestInput::from_json(&fallback.input).expect("failed to parse input");
    let output = TestOutput::from_json(&fallback.output).expect("failed to parse output");
    println!("keyvals count: {:?}", output.post_state.keyvals.len());

    // prints the state keys info and calculate the state root
    let mut kvs = Vec::new();
    for keyval in output.post_state.keyvals {
        let key = keyval.key.as_state_key();

        println!("key: 0x{}, info: {:?}", hex::encode(key), key.info());
        kvs.push((key, keyval.value));
    }

    let root = merkle::trie(&kvs, 0);
    println!("state root: 0x{}", hex::encode(root));
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
