//! Ed25519 signatures.
#![cfg(feature = "ed25519")]

pub use ed25519_zebra::{batch, Signature, SigningKey, VerificationKey, VerificationKeyBytes};
use rayon::{iter::ParallelIterator, slice::ParallelSlice};

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
        self.verifying.into()
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

/// Number of signatures per parallel chunk for batch verification.
const BATCH_PAR_CHUNK: usize = 32;

/// Batch verify a set of Ed25519 signatures.
///
/// Uses ZIP-215 batch verification (semantically identical to single-verify) and
/// parallelizes across rayon's pool for batches larger than [`BATCH_PAR_CHUNK`].
///
/// Returns `Ok(())` iff every `(message, signature, key)` triple is valid.
pub fn batch_verify(items: &[(&[u8], [u8; 64], [u8; 32])]) -> anyhow::Result<()> {
    if items.is_empty() {
        return Ok(());
    }
    if items.len() <= BATCH_PAR_CHUNK {
        return verify_one_batch(items);
    }

    items
        .par_chunks(BATCH_PAR_CHUNK)
        .try_for_each(verify_one_batch)
}

fn verify_one_batch(items: &[(&[u8], [u8; 64], [u8; 32])]) -> anyhow::Result<()> {
    use rand::rngs::OsRng;
    let mut batch = batch::Verifier::new();
    for (msg, sig, key) in items {
        batch.queue((
            VerificationKeyBytes::from(*key),
            Signature::from_bytes(sig),
            *msg,
        ));
    }
    batch.verify(OsRng).map_err(Into::into)
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
