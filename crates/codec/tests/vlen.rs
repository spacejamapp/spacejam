use jamcodec as codec;
use jamcodec::compact::vlen;
use serde::{Deserialize, Serialize};

#[test]
fn test_vlen() {
    // A large vector of auth output
    let data = [vec![176u8, 0], vec![0u8; 12288]].concat();
    let data_len = 12288;
    let prefix_len = 2;

    // Decode the length and prefix
    let (decoded_len, decoded_prefix_len) = vlen::decode_from(&data);
    assert_eq!(decoded_len, data_len as u64);
    assert_eq!(decoded_prefix_len, prefix_len);

    // Encode the length and prefix
    let encoded_len = vlen::encode(data_len as u64);
    assert_eq!(encoded_len, data[..2]);
}

#[derive(Debug, Serialize, Deserialize)]
struct VlenFoo {
    #[serde(with = "codec::vlen")]
    data: Vec<u8>,
}

#[test]
fn test_vlen_foo() {
    let foo = VlenFoo {
        data: vec![1, 2, 3],
    };
    let encoded = codec::encode(&foo).unwrap();
    let decoded = codec::decode::<VlenFoo>(&encoded).unwrap();
    assert_eq!(foo.data, decoded.data);
}
