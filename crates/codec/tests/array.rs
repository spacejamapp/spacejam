// Test array of fixed byte arrays and Vec<u8> serialization

use serde::{Deserialize, Serialize};
use serde_jam::bytes::array;
use serde_jam::{decode, encode};

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
    let encoded = serde_jam::encode(&test).unwrap();
    let decoded = serde_jam::decode::<Test>(&encoded).unwrap();
    assert_eq!(test.data, decoded.data);
}

#[test]
fn test_vec_u8_serialization() {
    let vec_data = vec![1u8, 2, 3, 4];

    let vec_encoded = encode(&vec_data).unwrap();
    println!(
        "Vec<u8> encoded: {:?} (len={})",
        vec_encoded,
        vec_encoded.len()
    );

    let vec_decoded: Vec<u8> = decode(&vec_encoded).unwrap();
    assert_eq!(vec_data, vec_decoded);
}

#[test]
fn test_slice_serialization() {
    let slice_data = &[1u8, 2, 3, 4][..];

    let slice_encoded = encode(&slice_data).unwrap();
    println!(
        "&[u8] encoded: {:?} (len={})",
        slice_encoded,
        slice_encoded.len()
    );

    let slice_decoded: Vec<u8> = decode(&slice_encoded).unwrap();
    assert_eq!(slice_data, slice_decoded);
}

#[test]
fn test_vec_vs_slice_comparison() {
    let data = [1u8, 2, 3, 4];
    let vec_data = data.to_vec();
    let slice_data = &data[..];

    let vec_encoded = encode(&vec_data).unwrap();
    let slice_encoded = encode(&slice_data).unwrap();

    println!(
        "Vec<u8> {:?} -> {:?} (len={})",
        vec_data,
        vec_encoded,
        vec_encoded.len()
    );
    println!(
        "&[u8]   {:?} -> {:?} (len={})",
        slice_data,
        slice_encoded,
        slice_encoded.len()
    );

    // Both should decode to the same Vec<u8>
    let vec_decoded: Vec<u8> = decode(&vec_encoded).unwrap();
    let slice_decoded: Vec<u8> = decode(&slice_encoded).unwrap();

    assert_eq!(vec_data, vec_decoded);
    assert_eq!(slice_data, slice_decoded);
}

#[test]
fn test_string_serialization() {
    let string_data = "hello world".to_string();

    let string_encoded = encode(&string_data).unwrap();
    println!(
        "String encoded: {:?} (len={})",
        string_encoded,
        string_encoded.len()
    );

    let string_decoded: String = decode(&string_encoded).unwrap();
    assert_eq!(string_data, string_decoded);
}

#[test]
fn test_str_serialization() {
    let str_data = "hello world";

    let str_encoded = encode(&str_data).unwrap();
    println!(
        "&str encoded: {:?} (len={})",
        str_encoded,
        str_encoded.len()
    );

    let str_decoded: String = decode(&str_encoded).unwrap();
    assert_eq!(str_data, str_decoded);
}
