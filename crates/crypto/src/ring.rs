//! Bandersnatch ring related primitives
#![cfg(feature = "vrf")]

use crate::vrf;
use ark_ec::AffineRepr;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_vrf::{
    suites::bandersnatch::{PcsParams, RingProofParams},
    Public,
};
use once_cell::sync::Lazy;

/// Number of keys in the ring.
///
/// TODO: add features to support full ring size
pub const RING_SIZE: usize = 6;

/// "Static" ring context data
pub static RING_CTX: Lazy<RingProofParams> = Lazy::new(|| {
    let buf = include_bytes!(
        "../bandersnatch-vrfs-spec/assets/example/data/zcash-srs-2-11-uncompressed.bin"
    );
    let pcs_params = PcsParams::deserialize_uncompressed_unchecked(&mut &buf[..])
        .expect("Failed to deserialize SRS parameters");
    RingProofParams::from_pcs_params(RING_SIZE, pcs_params).expect("Failed to create ring context")
});

/// Calculates the ring commitment for a set of Bandersnatch keys as per formula 6.1.3
/// Takes a vector of 32-byte Bandersnatch public keys and returns a ring commitment
pub fn commitment(keys: impl AsRef<[[u8; 32]]>) -> [u8; 144] {
    let keys = keys
        .as_ref()
        .iter()
        .map(|key| {
            AffineRepr::from_random_bytes(key).unwrap_or_else(|| {
                // If key is invalid (zeroed or can't be decoded), use padding point
                RingProofParams::padding_point()
            })
        })
        .collect::<Vec<_>>();

    let verifier_key = RING_CTX.verifier_key(&keys);
    let commitment = verifier_key.commitment();
    let mut bytes = [0u8; 144];
    commitment
        .serialize_compressed(bytes.as_mut_slice())
        .unwrap();
    bytes
}

/// Creates a VRF verifier for a set of Bandersnatch keys
pub fn verifier(keys: impl AsRef<[[u8; 32]]>) -> vrf::Verifier {
    let keys: Vec<_> = keys
        .as_ref()
        .iter()
        .map(|k| AffineRepr::from_random_bytes(k).unwrap_or_else(RingProofParams::padding_point))
        .map(Public)
        .collect();
    vrf::Verifier::new(keys)
}
