use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct Test {
    input: HashMap<String, String>,
    output: String,
}

#[test]
fn jam() {
    let test = include_str!("../jamtestvectors/trie/trie.json");
    let tests: Vec<Test> = serde_json::from_str(test).expect("failed to parse trie test");

    for test in tests {
        let root = trie::merkle(
            &test
                .input
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
        assert_eq!(hex::encode(root), test.output);
    }
}
