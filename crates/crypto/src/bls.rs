//! BLS utilities.

use w3f_bls::TinyBLS381;
pub use w3f_bls::{DoublePublicKeyScheme, SerializableToBytes};

/// BLS key pair.
pub type KeyPair = w3f_bls::Keypair<TinyBLS381>;

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
