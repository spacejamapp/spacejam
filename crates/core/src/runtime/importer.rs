//! Importer for SpaceJam

use crate::{
    block::{BlockInfo, Header},
    extrinsic::{TicketBody, TicketsOrKeys},
    runtime::{
        storage::{BlockStorage, Branch, KVStorage},
        tx, Config, Head, Runtime, Storage,
    },
    safrole::{Safrole, ValidatorData},
    state::key,
    Block, EntropyBuffer,
};

/// Importer for SpaceJam
pub struct Importer<'i, C: Config> {
    /// The runtime
    runtime: &'i Runtime<C>,
}

impl<'i, C: Config> Importer<'i, C> {
    /// Create a new importer
    pub fn new(runtime: &'i Runtime<C>) -> Self {
        Self { runtime }
    }

    /// Import the genesis block
    pub async fn import_genesis(
        &self,
        block: Block,
        validators: &[ValidatorData],
    ) -> anyhow::Result<()> {
        // 1. save the block to the storage
        self.runtime.storage.finalize(&block)?;

        // 2. initialize the recent blocks
        let recent: Vec<BlockInfo> = vec![block.header.clone().into()];
        self.runtime
            .storage
            .set(key::RECENT_BLOCKS, codec::encode(&recent)?)?;

        // 3. initialize the validator set
        let encoded = codec::encode(&validators)?;
        self.runtime
            .storage
            .set(key::PREVIOUS_VALIDATORS, encoded.clone())?;
        self.runtime
            .storage
            .set(key::CURRENT_VALIDATORS, encoded.clone())?;
        self.runtime.storage.set(key::NEXT_VALIDATORS, encoded)?;

        // 4. set the entropy
        let entropy = EntropyBuffer::default();
        self.runtime
            .storage
            .set(key::ENTROPY, codec::encode(&entropy)?)?;

        // 5. set the safrole state
        let safrole = Safrole {
            series: TicketsOrKeys::fallback(
                validators.iter().map(|v| v.bandersnatch).collect(),
                entropy,
            ),
            validators: validators.to_vec(),
            ..Default::default()
        };
        self.runtime
            .storage
            .set(key::SAFROLE, codec::encode(&safrole)?)?;

        // 5. initialize the grandpa state
        let mut grandpa = self.runtime.grandpa.write().await;
        grandpa.grid.next = validators.to_vec();
        grandpa.grid.curr = grandpa.grid.next.clone();
        grandpa.grid.prev = grandpa.grid.curr.clone();
        grandpa.finalize(block.header.clone(), None)?;
        drop(grandpa);

        Ok(())
    }

    /// Finalize blocks
    ///
    /// Note that we only store finalized blocks and the blocks authored
    /// by ourselves in our storage.
    #[tracing::instrument(skip_all, level = "debug", name = "Runtime::finalize")]
    pub async fn finalize(&self, block: Block) -> anyhow::Result<()> {
        let prev = self.runtime.grandpa.read().await.handshake.head.clone();
        tracing::debug!(
            "previous best block#{}: {}",
            prev.slot,
            hex::encode(prev.hash)
        );

        // 1. transit the global state
        tx::transit(
            block.clone(),
            &self.runtime.storage,
            &self.runtime.validator,
        )?;
        tracing::info!("Finalized block#{}", block.header.slot);

        // 2. save the block to the storage
        self.runtime.storage.save_block(&block)?;

        // 3. set the latest finalized head
        let head = Head {
            hash: block.hash()?,
            slot: block.header.slot,
        };
        self.runtime.storage.set_finalized(&head)?;

        // 4. drop the previous branch
        let branch = Branch::checkout(&self.runtime.storage, prev);
        branch.drop()?;

        // 5. update the grandpa state
        let next = if block.header.epoch_mark.is_some() {
            Some(self.runtime.storage.next_validators()?)
        } else {
            None
        };
        self.runtime
            .grandpa
            .write()
            .await
            .finalize(block.header.clone(), next)
    }

    /// Validate a block header.
    pub async fn validate(&self, header: &Header) -> anyhow::Result<()> {
        let handshake = self.runtime.grandpa.read().await.handshake.clone();
        let local_epoch = handshake.head.slot / crate::EPOCH_LENGTH;
        let remote_epoch = header.slot / crate::EPOCH_LENGTH;

        // if the epoch greater than the next, skip the validation.
        if local_epoch != 0 && remote_epoch > local_epoch + 1 {
            anyhow::bail!(
                "invalid epoch: local: {}, remote: {}",
                local_epoch,
                remote_epoch
            );
        }

        // present the verifying components
        let new_epoch = remote_epoch == local_epoch + 1;
        let slot = (header.slot % crate::EPOCH_LENGTH) as usize;
        let entropy_buffer = self.runtime.storage.entropy()?;
        let series = self.runtime.storage.safrole()?.series.clone();
        let mut entropy = entropy_buffer[3];
        let mut ticket: Option<TicketBody> = None;

        // handle entropy and attempt
        if new_epoch {
            entropy = entropy_buffer[2];

            // check the ticket mark
            if let Some(tickets) = header.tickets_mark {
                ticket = Some(tickets[slot]);
            }
        } else if let TicketsOrKeys::Tickets(tickets) = series {
            ticket = Some(tickets[slot]);
        }

        // indicate the keys to be used
        let keys = if new_epoch {
            self.runtime.storage.next_validators()?
        } else {
            self.runtime.storage.current_validators()?
        }
        .iter()
        .map(|v| v.bandersnatch)
        .collect::<Vec<_>>();

        // construct the message
        let mut oheader = header.clone();
        oheader.seal = [0; 96];
        oheader.entropy_source = [0; 96];
        let message = codec::encode(&oheader)?;

        // construct the context
        let mut context = Vec::new();
        if let Some(ticket) = ticket {
            context.extend_from_slice(&crate::JAM_TICKET_SEAL);
            context.extend_from_slice(&entropy);
            context.push(ticket.attempt);
        } else {
            context.extend_from_slice(&crate::JAM_FALLBACK_SEAL);
            context.extend_from_slice(&entropy);
        }

        // check the ticket seal
        let author_index = header.author_index;
        let verifier = crypto::ring::verifier(keys.clone());
        let output =
            verifier.ietf_vrf_verify(&message, &context, &header.seal, author_index as usize)?;
        if let Some(ticket) = ticket {
            if output != ticket.id {
                anyhow::bail!("header seal mismatch from ticket");
            }
        }

        Ok(())
    }
}
