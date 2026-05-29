//! Ed25519 signatures.
#![cfg(feature = "ed25519")]

pub use ed25519_zebra::{batch, Signature, SigningKey, VerificationKey, VerificationKeyBytes};
use rand::rngs::OsRng;
use rayon::{iter::ParallelIterator, slice::ParallelSlice};

/// Below this batch size, verify the whole slice as one ed25519 batch with
/// no rayon involvement.
const SEQUENTIAL_BATCH_THRESHOLD: usize = 32;

/// Chunk size when the batch is large enough to parallelize. Each chunk is
/// itself one ed25519 batch verification.
const PAR_CHUNK_SIZE: usize = 32;

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

/// Owned signature item for deferred verification.
pub struct SigItem {
    pub message: Vec<u8>,
    pub signature: [u8; 64],
    pub key: [u8; 32],
}

impl SigItem {
    /// Verify a single signature.
    pub fn verify(&self) -> anyhow::Result<()> {
        verify(&self.message, self.signature, self.key)
    }

    /// Batch verify a slice of items.
    pub fn batch_verify(items: &[Self]) -> anyhow::Result<()> {
        if items.is_empty() {
            return Ok(());
        }
        let view: Vec<(&[u8], [u8; 64], [u8; 32])> = items
            .iter()
            .map(|i| (i.message.as_slice(), i.signature, i.key))
            .collect();
        batch_verify(&view)
    }
}

/// Batch verify a set of Ed25519 signatures.
pub fn batch_verify(items: &[(&[u8], [u8; 64], [u8; 32])]) -> anyhow::Result<()> {
    if items.is_empty() {
        return Ok(());
    }
    if items.len() <= SEQUENTIAL_BATCH_THRESHOLD {
        return verify_batch(items);
    }

    items.par_chunks(PAR_CHUNK_SIZE).try_for_each(verify_batch)
}

fn verify_batch(items: &[(&[u8], [u8; 64], [u8; 32])]) -> anyhow::Result<()> {
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
