#![cfg(test)]

use crypto::shuffle;
use serde::{Deserialize, Serialize};

const TESTS: &[u8] = include_bytes!("../jamtestvectors/shuffle/shuffle_tests.json");

#[derive(Debug, Serialize, Deserialize)]
pub struct Test {
    pub input: u32,
    pub entropy: String,
    pub output: Vec<u32>,
}

#[test]
fn tests() -> anyhow::Result<()> {
    let tests: Vec<Test> = serde_json::from_slice(TESTS)?;

    for test in tests {
        let mut input = vec![0; test.input as usize];
        for i in 0..test.input as usize {
            input[i] = i as u32;
        }

        let entropy = hex::decode(test.entropy.trim_start_matches("0x"))
            .map_err(|e| anyhow::anyhow!("Failed to decode entropy: {e}"))?
            .try_into()
            .expect("entropy");

        let output = test.output;
        let result = shuffle::eq331(&input, entropy);
        assert_eq!(result, output, "Test {} failed", test.input);
    }

    Ok(())
}
