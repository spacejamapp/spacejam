//! Shuffle tests

use crypto::shuffle;
use serde::{Deserialize, Serialize};
use specjam::Registry;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestInput {
    input: u32,
    entropy: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TestOutput {
    output: Vec<u32>,
}

#[test]
fn tests() -> anyhow::Result<()> {
    // grab `shuffle_tests.json`
    let registry = Registry::new(PathBuf::from("../../res/jam-test-vectors"));
    let test = registry.shuffle()?.get(0)?;
    let input: Vec<TestInput> = serde_json::from_str(test.input.expect_json()?)?;
    let output: Vec<TestOutput> = serde_json::from_str(test.output.expect_json()?)?;

    for (source, target) in input.into_iter().zip(output) {
        let mut input = vec![0; source.input as usize];
        for i in 0..source.input as usize {
            input[i] = i as u32;
        }

        let entropy = hex::decode(source.entropy.trim_start_matches("0x"))
            .map_err(|e| anyhow::anyhow!("Failed to decode entropy: {e}"))?
            .try_into()
            .expect("entropy");

        let result = shuffle::eq331(&input, entropy);
        assert_eq!(result, target.output, "Test {} failed", source.input);
    }

    Ok(())
}
