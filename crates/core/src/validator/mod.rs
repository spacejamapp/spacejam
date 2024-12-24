//! Validator abstraction

use crate::{
    block::{Block, BlockInfo},
    state::{key, Storage},
    BandersnatchPublic, BlsPublic, Ed25519Public, ValidatorMetadata,
};
pub use {
    context::{Context, Patch},
    extrinsic::{ExtrinsicInMem, ExtrinsicInPool},
    public::{ValidatorData, ValidatorDataJson, Validators, ValidatorsData},
    result::{Error, Result, ValidationError},
    validate::{
        ValidateAssurance, ValidateDispute, ValidateExtrinsic, ValidateGuarantee, ValidatePreimage,
        ValidateTicket,
    },
};

mod context;
mod extrinsic;
mod public;
mod result;
mod validate;

/// Validator interface
pub trait Validator {
    /// BLS public key
    fn bls_public_key(&self) -> BlsPublic;

    /// Ed25519 public key
    fn ed25519_public_key(&self) -> Ed25519Public;

    /// Bandersnatch public key
    fn bandersnatch_public_key(&self) -> BandersnatchPublic;

    /// Metadata of the validator
    fn metadata(&self) -> ValidatorMetadata;

    /// Data of the validator
    fn data(&self) -> ValidatorData {
        ValidatorData {
            bls: self.bls_public_key(),
            ed25519: self.ed25519_public_key(),
            bandersnatch: self.bandersnatch_public_key(),
            metadata: self.metadata(),
        }
    }

    /// Mines a block
    fn mine(&self, block: BlockInfo, db: &impl Storage) -> anyhow::Result<Block> {
        let mut block = block.mine();

        // TODO: handle the transaction pool.
        block.header.extrinsic_hash = block.extrinsic.hash()?;
        block.header.slot = db.timeslot()?.unwrap_or(0) + 1;

        // write the new state to the database
        db.set(key::TIMESLOT, block.header.slot.to_le_bytes())?;
        Ok(block)
    }
}

impl Validator for () {
    fn bls_public_key(&self) -> BlsPublic {
        [0u8; 144]
    }

    fn ed25519_public_key(&self) -> Ed25519Public {
        [0u8; 32]
    }

    fn bandersnatch_public_key(&self) -> BandersnatchPublic {
        [0u8; 32]
    }

    fn metadata(&self) -> ValidatorMetadata {
        [0u8; 128]
    }
}
