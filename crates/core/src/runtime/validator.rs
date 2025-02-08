//! Validator abstraction

use crate::{
    safrole::ValidatorData, BandersnatchPublic, BandersnatchRingVrfSignature,
    BandersnatchVrfSignature, BlsPublic, Ed25519Public, OpaqueHash, ValidatorMetadata,
};
use anyhow::Result;

/// Validator interface
pub trait Validator {
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

    /// Bandersnatch output
    fn bandersnatch_output(&self, message: &[u8]) -> Result<BandersnatchVrfSignature>;

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

    /// Generate entropy from the given block header GP: (6.22)
    fn entropy(&self, entropy: OpaqueHash, source: &BandersnatchVrfSignature) -> Result<[u8; 32]> {
        let output = self.bandersnatch_output(source.as_ref())?;
        let mut input = entropy.to_vec();
        input.extend_from_slice(&output);
        Ok(crypto::blake2b(&input))
    }
}
