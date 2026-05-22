//! Spacejam's cryptographic primitives.

pub mod bls;
pub mod ed25519;
pub mod merkle;
pub mod ring;
pub mod shuffle;
pub mod vrf;

#[cfg(feature = "blake2")]
/// Compute the BLAKE2b 256-bit hash of a given input.
pub fn blake2b(input: &[u8]) -> [u8; 32] {
    let hash = blake2b_simd::Params::new()
        .hash_length(32)
        .to_state()
        .update(input)
        .finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(hash.as_bytes());
    out
}

#[cfg(feature = "blake3")]
/// Compute the BLAKE3 256-bit hash of a given input.
pub fn blake3(input: &[u8]) -> [u8; 32] {
    blake3::hash(input).into()
}

#[cfg(feature = "keccak")]
/// Compute the Keccak 256-bit hash of a given input.
pub fn keccak(input: &[u8]) -> [u8; 32] {
    use tiny_keccak::{Hasher, Keccak};

    let mut output = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(input);
    hasher.finalize(&mut output);
    output
}
