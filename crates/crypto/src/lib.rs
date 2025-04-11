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
    use blake2::{digest::consts::U32, Blake2b, Digest};

    let mut hasher = Blake2b::<U32>::new();
    hasher.update(input);
    hasher.finalize().into()
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
