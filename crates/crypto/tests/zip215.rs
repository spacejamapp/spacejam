#![cfg(feature = "ed25519")]

use serde::{Deserialize, Serialize};
use spacejam_crypto::ed25519;

const VECTORS_FILE: &str = "../../res/jam-conformance/crypto/ed25519/vectors.json";

#[derive(Serialize, Deserialize)]
struct TestVector {
    /// Index of the test case
    number: u8,
    /// Description of the test case
    desc: String,
    /// Public key A (32 bytes hex)
    pk: String,
    /// Commitment R (32 bytes hex)
    r: String,
    /// Scalar s (32 bytes hex, always 0 in our case)
    s: String,
    /// Message (hex)
    msg: String,
    /// Whether A encoding is canonical
    pk_canonical: bool,
    /// Whether R encoding is canonical
    r_canonical: bool,
}

/// Decoded test vector components
struct DecodedTestVector {
    vk_array: [u8; 32],
    sig_bytes: [u8; 64],
    message: Vec<u8>,
}

/// Load test vectors from JSON file
fn load_test_vectors() -> Vec<TestVector> {
    let json_data = std::fs::read_to_string(VECTORS_FILE)
        .expect("Failed to read {VECTORS_FILE}. Run the binary first to generate it.");
    serde_json::from_str(&json_data).expect("Failed to parse {VECTORS_FILE}")
}

/// Decode a test vector into byte arrays
fn decode_test_vector(tv: &TestVector) -> DecodedTestVector {
    let vk_bytes = hex::decode(&tv.pk).expect("Invalid public key hex");
    let r_bytes = hex::decode(&tv.r).expect("Invalid R hex");
    let s_bytes = hex::decode(&tv.s).expect("Invalid s hex");
    let message = hex::decode(&tv.msg).expect("Invalid message hex");

    // Construct the signature: sig = R || s (64 bytes)
    let mut sig_bytes = [0u8; 64];
    sig_bytes[0..32].copy_from_slice(&r_bytes);
    sig_bytes[32..64].copy_from_slice(&s_bytes);

    // Construct the verification key (32 bytes)
    let mut vk_array = [0u8; 32];
    vk_array.copy_from_slice(&vk_bytes);

    DecodedTestVector {
        vk_array,
        sig_bytes,
        message,
    }
}

#[test]
fn ed25519() {
    let test_vectors = load_test_vectors();
    for tv in test_vectors {
        let decoded = decode_test_vector(&tv);
        ed25519::verify(&decoded.message, decoded.sig_bytes, decoded.vk_array).unwrap();
    }
}
