//! Trie tests

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct TestInput {
    input: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct TestOutput {
    output: String,
}

#[test]
fn jam() {
    let test = specjam::registry::tests::TEST_TRIE_TRIE;
    let tests: Vec<TestInput> =
        serde_json::from_str(&test.input).expect("failed to parse trie test input");
    let output: Vec<TestOutput> =
        serde_json::from_str(&test.output).expect("failed to parse trie test output");

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
