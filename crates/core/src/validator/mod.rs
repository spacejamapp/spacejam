//! Validator abstraction

use crate::{
    block::{Block, BlockInfo},
    extrinsic::TicketsOrKeys,
    state::{key, Storage},
    BandersnatchPublic, BandersnatchVrfSignature, BlsPublic, Ed25519Public, ValidatorMetadata,
    JAM_ENTROPY, JAM_FALLBACK_SEAL, JAM_TICKET_SEAL,
};
pub use {
    context::{Context, Patch},
    extrinsic::{ExtrinsicInMem, ExtrinsicInPool},
    public::{ValidatorData, ValidatorDataJson, Validators, ValidatorsData},
    result::{Error, Result, ValidationError},
    validate::ValidateExtrinsic,
};

mod context;
mod extrinsic;
mod public;
mod result;
mod validate;

/// Validator interface
pub trait Validator: TryFrom<String> {
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
    ) -> anyhow::Result<BandersnatchVrfSignature>;

    /// Bandersnatch output
    fn bandersnatch_output(
        &self,
        sig: BandersnatchVrfSignature,
    ) -> anyhow::Result<BandersnatchVrfSignature>;

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
        let entropy = db.entropy()?.unwrap_or_default();
        let safrole = db.safrole()?.unwrap_or_default();
        let timeslot = db.timeslot()?.unwrap_or(0);

        // TODO: handle the transaction pool.
        block.header.extrinsic_hash = block.extrinsic.hash()?;
        block.header.slot = timeslot + 1;

        // NOTE: get validators of the block's timeslot
        //
        // TODO: use next epoch's validators is new epoch is starting
        let keys: Vec<[u8; 32]> = db
            .current_validators()?
            .unwrap_or_default()
            .into_iter()
            .map(|v| v.bandersnatch)
            .collect();

        let message = codec::encode(&block)?;
        block.header.seal = match safrole.series {
            TicketsOrKeys::Tickets(tickets) => {
                let entry_index = tickets
                    .iter()
                    .enumerate()
                    .find(|(_, t)| t.attempt as u32 == timeslot)
                    .map(|(i, _)| i)
                    .unwrap_or_default();
                let mut context = JAM_TICKET_SEAL.to_vec();
                context.extend_from_slice(&entropy[3]);
                context.push(entry_index as u8);
                self.bandersnatch_sign(&keys, &context, &message)?
            }
            TicketsOrKeys::Keys(_) => {
                let mut context = JAM_FALLBACK_SEAL.to_vec();
                context.extend_from_slice(&entropy[3]);
                self.bandersnatch_sign(&keys, &context, &message)?
            }
        };

        block.header.entropy_source = {
            let mut context = JAM_ENTROPY.to_vec();
            context.extend_from_slice(&self.bandersnatch_output(block.header.seal)?);
            self.bandersnatch_sign(&keys, &context, &[])?
        };

        // write the new state to the database
        //
        // TODO: mb not, store it in a separate database.
        db.set(key::TIMESLOT, block.header.slot.to_le_bytes())?;
        Ok(block)
    }
}
