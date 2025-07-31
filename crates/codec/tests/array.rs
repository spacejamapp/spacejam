// Test array of fixed byte arrays

use jamcodec::bytes::array;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Test {
    #[serde(with = "array")]
    data: Vec<[u8; 64]>,
}

#[test]
fn codec() {
    let test = Test {
        data: vec![[0; 64], [1; 64]],
    };
    let encoded = jamcodec::encode(&test).unwrap();
    let decoded = jamcodec::decode::<Test>(&encoded).unwrap();
    assert_eq!(test.data, decoded.data);
}
