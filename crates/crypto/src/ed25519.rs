//! Ed25519 signatures.
#![cfg(feature = "ed25519")]

pub use ed25519_zebra::{Signature, SigningKey, VerificationKey};

/// Ed25519 key pair.
#[derive(Clone)]
pub struct KeyPair {
    /// Signing key.
    pub signing: SigningKey,

    /// Verifying key.
    pub verifying: VerificationKey,
}

impl KeyPair {
    /// Get the public key.
    pub fn public(&self) -> [u8; 32] {
        self.verifying.clone().into()
    }
}

impl From<[u8; 32]> for KeyPair {
    fn from(seed: [u8; 32]) -> Self {
        let signing = SigningKey::from_bytes(&seed);
        let verifying = VerificationKey::from(&signing);
        Self { signing, verifying }
    }
}

/// Verify an Ed25519 signature.
pub fn verify(message: &[u8], signature: [u8; 64], key: [u8; 32]) -> anyhow::Result<()> {
    let key = VerificationKey::try_from(key)?;
    let signature = Signature::from_bytes(&signature);
    key.verify(&signature, message).map_err(Into::into)
}

#[cfg(feature = "rand")]
impl Default for KeyPair {
    fn default() -> Self {
        use rand::{rngs::OsRng, Rng};
        let seed = OsRng.gen::<[u8; 32]>();
        Self::from(seed)
    }
}

#[cfg(feature = "tls")]
mod tls {
    use super::KeyPair;
    use ed25519_zebra::ed25519::pkcs8::EncodePrivateKey;

    impl KeyPair {
        /// Get the pkcs8 encoded public key.
        pub fn private_pkcs8_der(&self) -> Result<Vec<u8>, anyhow::Error> {
            let der = self.signing.to_pkcs8_der()?;
            Ok(der.as_bytes().to_vec())
        }
    }

    impl rcgen::RemoteKeyPair for KeyPair {
        fn public_key(&self) -> &[u8] {
            self.verifying.as_ref()
        }

        fn sign(&self, message: &[u8]) -> Result<Vec<u8>, rcgen::Error> {
            let signature = self.signing.sign(message);
            Ok(signature.to_bytes().to_vec())
        }

        fn algorithm(&self) -> &'static rcgen::SignatureAlgorithm {
            &rcgen::PKCS_ED25519
        }
    }
}
