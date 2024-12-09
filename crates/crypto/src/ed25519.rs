//! Ed25519 signatures.

use ed25519_dalek::{Signature, VerifyingKey};

/// Verify an Ed25519 signature.
pub fn verify(message: &[u8], signature: [u8; 64], key: [u8; 32]) -> anyhow::Result<()> {
    let key = VerifyingKey::from_bytes(&key)?;
    let signature = Signature::from_bytes(&signature);
    key.verify_strict(message, &signature).map_err(Into::into)
}
