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
    data: Vec<u8>,
}

#[test]
fn test_vlen_foo() {
    let foo = VlenFoo {
        data: vec![1, 2, 3],
    };
    let encoded = codec::encode(&foo).unwrap();
    assert_eq!(encoded, vec![3, 1, 2, 3]);

    let decoded = codec::decode::<VlenFoo>(&encoded).unwrap();
    assert_eq!(foo.data, decoded.data);
}

#[test]
fn test_vlen_foo_large() {
    let foo = VlenFoo {
        data: vec![0u8; 12288],
    };
    let encoded = codec::encode(&foo).unwrap();
    assert_eq!(encoded.len(), 12290);
    assert_eq!(encoded[..2], vec![176, 0]);

    let decoded = codec::decode::<VlenFoo>(&encoded).unwrap();
    assert_eq!(foo.data, decoded.data);
}

#[test]
fn just_vec() {
    let vec = vec![1u8, 2, 3];
    let encoded = codec::encode(&vec).unwrap();
    assert_eq!(encoded, vec![3, 1, 2, 3]);

    let decoded: Vec<u8> = codec::decode(&encoded).unwrap();
    assert_eq!(decoded, vec![1u8, 2, 3]);
}
