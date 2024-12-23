//! BLS12-381 public key.

use ark_bls12_381::{Bls12_381, Fr as Bls12_381Scalar, G1Affine, G2Affine, G2Projective};
use ark_ec::{pairing::Pairing, AffineRepr, CurveGroup, PrimeGroup};
use ark_ff::PrimeField;
use ark_serialize::CanonicalDeserialize;

#[allow(unused)]
#[derive(Clone)]
pub struct PublicKey(pub(crate) G1Affine);

impl PublicKey {
    /// Verify a signature.
    pub fn verify(&self, message: &[u8], signature: &[u8; 96]) -> bool {
        let message_hash =
            G2Projective::generator() * Bls12_381Scalar::from_le_bytes_mod_order(message);
        let signature = match G2Affine::deserialize_compressed(&signature[..]) {
            Ok(sig) => sig,
            Err(_) => return false,
        };

        let lhs = Bls12_381::pairing(self.0, message_hash.into_affine());
        let rhs = Bls12_381::pairing(G1Affine::generator(), signature);
        lhs == rhs
    }
}
