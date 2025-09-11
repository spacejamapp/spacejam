//! Bandersnatch ring related primitives
#![cfg(feature = "vrf")]

use crate::vrf;
use ark_ec::AffineRepr;
use ark_serialize::CanonicalDeserialize;
use ark_vrf::{suites::bandersnatch::RingProofParams, Public};
use once_cell::sync::Lazy;

/// Number of keys in the ring.
///
/// TODO: add features to support full ring size
pub const RING_SIZE: usize = 6;

/// "Static" ring context data
pub static RING_CTX: Lazy<RingProofParams> = Lazy::new(|| {
    let buf =
        include_bytes!("../bandersnatch-vrfs-spec/assets/example/data/size-6-with-zcash-srs.bin");
    RingProofParams::deserialize_uncompressed_unchecked(&mut &buf[..])
        .expect("Failed to deserialize SRS parameters")
});

/// Calculates the ring commitment for a set of Bandersnatch keys as per formula 6.1.3
/// Takes a vector of 32-byte Bandersnatch public keys and returns a ring commitment
pub fn commitment(keys: impl AsRef<[[u8; 32]]>) -> [u8; 144] {
    self::verifier(keys).commitment()
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
