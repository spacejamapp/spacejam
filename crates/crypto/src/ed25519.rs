//! Ed25519 signatures.

use ed25519_dalek::Signer;
pub use ed25519_dalek::{Signature, SigningKey, VerifyingKey};
use rcgen::RemoteKeyPair;

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

impl RemoteKeyPair for KeyPair {
    fn public_key(&self) -> &[u8] {
        self.verifying.as_bytes().as_ref()
    }

    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, rcgen::Error> {
        let signature = self.signing.sign(message);
        Ok(signature.to_bytes().to_vec())
    }

    fn algorithm(&self) -> &'static rcgen::SignatureAlgorithm {
        &rcgen::PKCS_ED25519
    }
}

/// Verify an Ed25519 signature.
pub fn verify(message: &[u8], signature: [u8; 64], key: [u8; 32]) -> anyhow::Result<()> {
    let key = VerifyingKey::from_bytes(&key)?;
    let signature = Signature::from_bytes(&signature);
    key.verify_strict(message, &signature).map_err(Into::into)
}
