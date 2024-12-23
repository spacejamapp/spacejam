//! BLS12-381 secret key.

use crate::bls::{PublicKey, Signature};
use ark_bls12_381::{Fr as Bls12_381Scalar, G1Projective, G2Projective};
use ark_ec::{CurveGroup, PrimeGroup};
use ark_ff::{PrimeField, UniformRand};
use ark_std::rand;

// Define structures for our key pair
#[derive(Clone)]
pub struct SecretKey(pub(crate) Bls12_381Scalar);

impl SecretKey {
    /// Generate a random private key.
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self(Bls12_381Scalar::rand(&mut rng))
    }

    /// Sign a message.
    pub fn sign(&self, message: &[u8]) -> Signature {
        let message_hash =
            G2Projective::generator() * Bls12_381Scalar::from_le_bytes_mod_order(message);
        let signature = message_hash * self.0;
        Signature(signature)
    }

    /// Get the public key from the private key.
    pub fn public(&self) -> PublicKey {
        PublicKey((G1Projective::generator() * self.0).into_affine())
    }
}
