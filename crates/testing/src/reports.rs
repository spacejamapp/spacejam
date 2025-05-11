#![cfg(test)]

use runtime::tx::guarantee::{
    error::{Error, Result},
    State, StateJson,
};
use serde::{Deserialize, Serialize};
use spacejson::{Json, ResultJson};
pub use types::*;

/// Test input.
#[derive(Debug, Clone, Serialize, Deserialize, Json)]
pub struct TestInput {
    #[json(nested)]
    pub input: Input,
    #[json(nested)]
    pub pre_state: State,
}

/// Test output.
#[derive(Debug, Serialize, Deserialize, Json, Clone)]
pub struct TestOutput {
    #[json(ResultJson<OutputJson, Error>)]
    pub output: Result<Output>,
    #[json(nested)]
    pub post_state: State,
}

mod types {
    use score::{
        extrinsic::{GuaranteesExtrinsic, ReportGuaranteeJson},
        service::{ReportedWorkPackage, ReportedWorkPackageJson},
        Block, Ed25519Public, OpaqueHash, TimeSlot,
    };
    use serde::{Deserialize, Serialize};
    use spacejson::Json;

    /// Input of the reporting module.
    #[derive(Debug, Clone, Serialize, Deserialize, Json)]
    pub struct Input {
        pub slot: TimeSlot,
        #[json(Vec<ReportGuaranteeJson>)]
        pub guarantees: GuaranteesExtrinsic,
        #[json(Vec<String>)]
        pub known_packages: Vec<OpaqueHash>,
    }

    impl From<Input> for Block {
        fn from(value: Input) -> Self {
            let mut block = Block::default();
            block.header.slot = value.slot;
            block.extrinsic.guarantees = value.guarantees;
            block
        }
    }

    /// Output of the reporting module.
    #[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
    pub struct Output {
        #[json(nested)]
        pub reported: Vec<ReportedWorkPackage>,
        #[json(Vec<String>)]
        pub reporters: Vec<Ed25519Public>,
    }
}

include!(concat!(env!("OUT_DIR"), "/reports.rs"));

/// The big report bin
const BIG_REPORT_BIN: &[u8] =
    include_bytes!("../../../res/jam-test-vectors/reports/tiny/big_work_report_output-1.bin");

const BIG_REPORT_JSON: &str =
    include_str!("../../../res/jam-test-vectors/reports/tiny/big_work_report_output-1.json");

#[derive(Debug, Serialize, Deserialize, Json, Clone)]
struct TestFile {
    #[json(nested)]
    pub input: Input,
    #[json(nested)]
    pub pre_state: State,
    #[json(ResultJson<OutputJson, Error>)]
    pub output: Result<Output>,
    #[json(nested)]
    pub post_state: State,
}

#[ignore]
#[test]
fn test_big_reports_codec() {
    let decoded = serde_json::from_str::<TestFileJson>(BIG_REPORT_JSON).unwrap();
    let test_file = TestFile::try_from(decoded.clone()).unwrap();
    let encoded = codec::encode(&test_file.input.guarantees[0].report).unwrap();

    let spacejam = [
        encoded[..12558].to_vec(),
        [0].to_vec(),
        encoded[12559..].to_vec(),
    ]
    .concat();
    let polkajam = BIG_REPORT_BIN[1..spacejam.len() + 1].to_vec();

    // ensure the encoded bytes are same as expected
    spacejam
        .iter()
        .zip(polkajam.iter())
        .enumerate()
        .for_each(|(i, (a, b))| {
            assert_eq!(a, b, "at index {}, total length {}", i, spacejam.len());
        });
}
