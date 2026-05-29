//! Trie tests

use crypto::merkle;
use serde::{Deserialize, Serialize};
use specjam::Registry;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct TestInput {
    input: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct TestOutput {
    output: String,
}

#[test]
fn trie() {
    let registry = Registry::new(PathBuf::from("../../res/jam-test-vectors"));
    let test = registry.trie().unwrap().get(0).unwrap();

    let tests: Vec<TestInput> =
        serde_json::from_str(test.input.expect_json().expect("trie input must be JSON"))
            .expect("failed to parse trie test input");
    let output: Vec<TestOutput> =
        serde_json::from_str(test.output.expect_json().expect("trie output must be JSON"))
            .expect("failed to parse trie test output");

    for (input, output) in tests
        .into_iter()
        .map(|i| i.input)
        .zip(output.into_iter().map(|o| o.output))
    {
        let root = merkle::trie(
            &input
                .into_iter()
                .map(|(k, v)| {
                    (
                        hex::decode(k)
                            .expect("failed to decode key")
                            .try_into()
                            .expect("failed to convert to bytes32"),
                        hex::decode(v).expect("failed to decode value"),
                    )
                })
                .collect::<Vec<_>>(),
            0,
        );
        assert_eq!(hex::encode(root), output);
    }
}
