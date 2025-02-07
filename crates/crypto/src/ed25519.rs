//! Ed25519 signatures.

pub use ed25519_dalek::{Signature, SigningKey, VerifyingKey};

/// Ed25519 key pair.
#[derive(Clone)]
pub struct KeyPair {
    /// Signing key.
    pub signing: SigningKey,

    /// Verifying key.
    pub verifying: VerifyingKey,
}

impl From<[u8; 32]> for KeyPair {
    fn from(seed: [u8; 32]) -> Self {
        let signing = SigningKey::from_bytes(&seed);
        let verifying = VerifyingKey::from(&signing);
        Self { signing, verifying }
    }
}

/// Verify an Ed25519 signature.
pub fn verify(message: &[u8], signature: [u8; 64], key: [u8; 32]) -> anyhow::Result<()> {
    let key = VerifyingKey::from_bytes(&key)?;
    let signature = Signature::from_bytes(&signature);
    key.verify_strict(message, &signature).map_err(Into::into)
}
