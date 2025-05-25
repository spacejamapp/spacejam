use crate::{
    BandersnatchPublic, BandersnatchRingVrfSignature, BandersnatchVrfSignature, BlsPublic,
    Ed25519Public, ValidatorMetadata, runtime::Validator,
};
use crypto::{bls, ed25519, vrf};

/// A testing validator
pub struct TestValidator {
    /// BLS key pair.
    pub bls: bls::KeyPair,

    /// Ed25519 key pair.
    pub ed25519: ed25519::KeyPair,

    /// Banersnatch key pair.
    pub banersnatch: vrf::KeyPair,
}

impl Validator for TestValidator {
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
        self.banersnatch.ietf_sign(keys.to_vec(), message, context)
    }

    fn bandersnatch_ring_sign(
        &self,
        keys: &[[u8; 32]],
        context: &[u8],
        message: &[u8],
    ) -> anyhow::Result<BandersnatchRingVrfSignature> {
        self.banersnatch.ring_sign(keys.to_vec(), message, context)
    }

    fn metadata(&self) -> ValidatorMetadata {
        [0u8; 128]
    }

    fn ed25519(&self) -> Option<ed25519::KeyPair> {
        Some(self.ed25519.clone())
    }
}

impl From<[u8; 32]> for TestValidator {
    fn from(seed: [u8; 32]) -> Self {
        Self {
            bls: bls::KeyPair::from(seed),
            ed25519: ed25519::KeyPair::from(seed),
            banersnatch: vrf::KeyPair::from(seed),
        }
    }
}
