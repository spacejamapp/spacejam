//! Validator from local.

use anyhow::Result;
use crypto::{bls, ed25519, vrf};
use rand::Rng;
use score::{
    validator::Validator, BandersnatchPublic, BandersnatchVrfSignature, BlsPublic, Ed25519Public,
    ValidatorMetadata,
};

/// Validator from local.
pub struct LocalValidator {
    /// BLS key pair.
    pub bls: bls::KeyPair,

    /// Ed25519 key pair.
    pub ed25519: ed25519::KeyPair,

    /// Banersnatch key pair.
    pub banersnatch: vrf::KeyPair,
}

impl Default for LocalValidator {
    fn default() -> Self {
        Self::from([0u8; 32])
    }
}

impl LocalValidator {
    /// Create a new local validator.
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let seed: [u8; 32] = rng.gen();
        seed.into()
    }
}

impl Validator for LocalValidator {
    fn bls_public_key(&self) -> BlsPublic {
        self.bls.public()
    }

    fn ed25519_public_key(&self) -> Ed25519Public {
        *self.ed25519.verifying.as_bytes()
    }

    fn bandersnatch_public_key(&self) -> BandersnatchPublic {
        self.banersnatch
            .public()
            .expect("invalid bandersnatch public key")
    }

    fn bandersnatch_sign(
        &self,
        keys: &[[u8; 32]],
        context: &[u8],
        message: &[u8],
    ) -> anyhow::Result<BandersnatchVrfSignature> {
        self.banersnatch.sign(keys.to_vec(), context, message)
    }

    fn bandersnatch_output(
        &self,
        signature: BandersnatchVrfSignature,
    ) -> Result<BandersnatchVrfSignature> {
        self.banersnatch.output(&signature)
    }

    fn metadata(&self) -> ValidatorMetadata {
        [0u8; 128]
    }
}

impl From<[u8; 32]> for LocalValidator {
    fn from(seed: [u8; 32]) -> Self {
        Self {
            bls: bls::KeyPair::from(seed),
            ed25519: ed25519::KeyPair::from(seed),
            banersnatch: vrf::KeyPair::from(seed),
        }
    }
}
