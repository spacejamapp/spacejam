use crate::vrf;
use ark_ec_vrfs::{
    prelude::{
        ark_ec::AffineRepr,
        ark_serialize::{CanonicalDeserialize, CanonicalSerialize},
    },
    suites::bandersnatch::edwards::{BandersnatchSha512Ell2, PcsParams, RingContext},
    AffinePoint, Public,
};
use once_cell::sync::Lazy;

/// Number of keys in the ring.
///
/// TODO: add features to support full ring size
pub const RING_SIZE: usize = 6;

/// "Static" ring context data
pub static RING_CTX: Lazy<RingContext> = Lazy::new(|| {
    let buf = include_bytes!(
        "../bandersnatch-vrfs-spec/assets/example/data/zcash-srs-2-11-uncompressed.bin"
    );
    let pcs_params = PcsParams::deserialize_uncompressed_unchecked(&mut &buf[..])
        .expect("Failed to deserialize SRS parameters");
    RingContext::from_srs(RING_SIZE, pcs_params).expect("Failed to create ring context")
});

/// Calculates the ring commitment for a set of Bandersnatch keys as per formula 6.1.3
/// Takes a vector of 32-byte Bandersnatch public keys and returns a ring commitment
pub fn commitment(keys: Vec<[u8; 32]>) -> [u8; 144] {
    let pubkeys: Vec<AffinePoint<BandersnatchSha512Ell2>> = keys
        .iter()
        .filter_map(|k| AffineRepr::from_random_bytes(k))
        .collect();
    println!("pubkeys: {:?}", pubkeys.len());
    let verifier_key = RING_CTX.verifier_key(&pubkeys);
    let commitment = verifier_key.commitment();

    let mut bytes = [0u8; 144];
    commitment
        .serialize_compressed(bytes.as_mut_slice())
        .unwrap();
    bytes
}

/// Creates a VRF verifier for a set of Bandersnatch keys
pub fn verifier(keys: Vec<[u8; 32]>) -> vrf::Verifier {
    let keys: Vec<_> = keys
        .iter()
        .filter_map(|k| AffineRepr::from_random_bytes(k).map(Public))
        .collect();
    vrf::Verifier::new(keys)
}
