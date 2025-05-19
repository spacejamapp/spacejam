//! Validator abstraction

use anyhow::Result;
use score::{
    BandersnatchPublic, BandersnatchRingVrfSignature, BandersnatchVrfSignature, BlsPublic,
    Ed25519Public, ValidatorMetadata, safrole::ValidatorData,
};

/// Validator interface
pub trait Validator {
    /// Get the development validator
    fn dev() -> Self;

    /// Random validator
    fn random() -> Self;

    /// BLS public key
    fn bls_public_key(&self) -> BlsPublic;

    /// Ed25519 public key
    fn ed25519_public_key(&self) -> Ed25519Public;

    /// Bandersnatch public key
    fn bandersnatch_public_key(&self) -> BandersnatchPublic;

    /// Bandersnatch sign
    fn bandersnatch_sign(
        &self,
        keys: &[[u8; 32]],
        context: &[u8],
        message: &[u8],
    ) -> Result<BandersnatchVrfSignature>;

    /// Bandersnatch ring sign
    fn bandersnatch_ring_sign(
        &self,
        keys: &[[u8; 32]],
        context: &[u8],
        message: &[u8],
    ) -> Result<BandersnatchRingVrfSignature>;

    /// Metadata of the validator
    fn metadata(&self) -> ValidatorMetadata;

    /// Ed25519 key pair
    fn ed25519(&self) -> Option<crypto::ed25519::KeyPair> {
        None
    }

    /// Data of the validator
    fn data(&self) -> ValidatorData {
        ValidatorData {
            bls: self.bls_public_key(),
            ed25519: self.ed25519_public_key(),
            bandersnatch: self.bandersnatch_public_key(),
            metadata: self.metadata(),
        }
    }
}

impl Validator for crypto::ed25519::KeyPair {
    fn dev() -> Self {
        Self::from([0; 32])
    }

    fn random() -> Self {
        unimplemented!()
    }

    fn ed25519_public_key(&self) -> Ed25519Public {
        *self.verifying.as_bytes()
    }

    fn bls_public_key(&self) -> BlsPublic {
        [0; 144]
    }

    fn bandersnatch_public_key(&self) -> BandersnatchPublic {
        [0; 32]
    }

    fn bandersnatch_sign(
        &self,
        _keys: &[[u8; 32]],
        _context: &[u8],
        _message: &[u8],
    ) -> Result<BandersnatchVrfSignature> {
        Ok([0; 96])
    }

    fn bandersnatch_ring_sign(
        &self,
        _keys: &[[u8; 32]],
        _context: &[u8],
        _message: &[u8],
    ) -> Result<BandersnatchRingVrfSignature> {
        Ok([0; 784])
    }

    fn metadata(&self) -> ValidatorMetadata {
        [0; 128]
    }

    fn ed25519(&self) -> Option<crypto::ed25519::KeyPair> {
        Some(self.clone())
    }
}
