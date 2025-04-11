//! BLS utilities.
#![cfg(feature = "bls")]

use std::ops::Deref;
use w3f_bls::TinyBLS381;
pub use w3f_bls::{DoublePublicKeyScheme, SerializableToBytes};

/// BLS secret key.
pub type SecretKey = w3f_bls::SecretKey<TinyBLS381>;

/// BLS public key.
pub type PublicKey = w3f_bls::PublicKey<TinyBLS381>;

/// BLS signature.
pub type Signature = w3f_bls::Signature<TinyBLS381>;

/// BLS double public key.
pub type DoublePublicKey = w3f_bls::DoublePublicKey<TinyBLS381>;

/// BLS double signature.
pub type DoubleSignature = w3f_bls::DoubleSignature<TinyBLS381>;

/// BLS key pair.
pub struct KeyPair {
    inner: w3f_bls::Keypair<TinyBLS381>,
    public: DoublePublicKey,
}

impl KeyPair {
    /// Get the public key as a 32-byte array.
    pub fn public(&self) -> [u8; 144] {
        let mut buf = [0; 144];
        buf.copy_from_slice(&self.public.to_bytes());
        buf
    }
}

impl From<[u8; 32]> for KeyPair {
    fn from(seed: [u8; 32]) -> Self {
        let bls_sk = SecretKey::from_seed(&seed);
        let bls_pk = bls_sk.into_public();
        let pair = w3f_bls::Keypair {
            secret: bls_sk,
            public: bls_pk,
        };
        let public = pair.into_double_public_key();

        Self {
            inner: pair,
            public,
        }
    }
}

impl Deref for KeyPair {
    type Target = w3f_bls::Keypair<TinyBLS381>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
