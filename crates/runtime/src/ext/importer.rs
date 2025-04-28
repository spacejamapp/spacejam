//! Importer for SpaceJam

use crate::{
    Config, Hook, Runtime, Storage,
    storage::{KVStorage, SyncStorage},
    tx,
};
use score::{
    Block, EntropyBuffer,
    block::{BlockInfo, Header},
    extrinsic::{TicketBody, TicketsOrKeys},
    safrole::{Safrole, ValidatorData},
    state::key,
};

impl<C: Config> Runtime<C> {
    /// Import the genesis block
    pub async fn import_genesis(
        &self,
        block: Block,
        validators: &[ValidatorData],
    ) -> anyhow::Result<()> {
        // 1. save the block to the storage
        self.storage.set_block(&block)?;

        // 2. initialize the recent blocks
        let recent: Vec<BlockInfo> = vec![block.header.clone().into()];
        self.storage
            .set(key::RECENT_BLOCKS, codec::encode(&recent)?)?;

        // 3. initialize the validator set
        let encoded = codec::encode(&validators)?;
        self.storage
            .set(key::PREVIOUS_VALIDATORS, encoded.clone())?;
        self.storage.set(key::CURRENT_VALIDATORS, encoded.clone())?;
        self.storage.set(key::NEXT_VALIDATORS, encoded)?;

        // 4. set the entropy
        let entropy = EntropyBuffer::default();
        self.storage.set(key::ENTROPY, codec::encode(&entropy)?)?;

        // 5. set the safrole state
        let series =
            TicketsOrKeys::fallback(validators.iter().map(|v| v.bandersnatch).collect(), entropy);
        let safrole = Safrole {
            series: series.clone(),
            validators: validators.to_vec(),
            ..Default::default()
        };
        self.storage.set(key::SAFROLE, codec::encode(&safrole)?)?;

        // 5. initialize the grandpa state
        let mut grandpa = self.grandpa.write().await;
        grandpa.grid.next = validators.to_vec();
        grandpa.grid.curr = grandpa.grid.next.clone();
        grandpa.grid.prev = grandpa.grid.curr.clone();
        grandpa.finalize(block.header.clone(), None)?;

        Ok(())
    }

    /// Finalize blocks
    ///
    /// Note that we only store finalized blocks and the blocks authored
    /// by ourselves in our storage.
    pub async fn finalize(&self, block: Block) -> anyhow::Result<()> {
        let prev = self.grandpa.read().await.handshake.head.clone();

        if block.header.parent != prev.hash {
            anyhow::bail!(
                "invalid parent: 0x{} != 0x{}",
                hex::encode(block.header.parent[..3].as_ref()),
                hex::encode(prev.hash[..3].as_ref())
            );
        }

        // 1. transit the global state
        let hash = block.header.hash()?;
        tx::transit::<C::Vm>(block.clone(), &self.storage, &self.validator)?;
        tracing::info!(
            "finalized block#{}@{}, previous block#{}@{}",
            block.header.slot,
            hex::encode(&hash[..3]),
            prev.slot,
            hex::encode(prev.hash[..3].as_ref())
        );

        // 2. save the block to the storage
        self.storage.set_block(&block)?;
        if let Some(series) = block.header.tickets_mark {
            tracing::info!(
                "next tickets: {:#?}",
                series.iter().map(|t| hex::encode(t.id)).collect::<Vec<_>>()
            );
            self.storage.set_next_series(&series)?;
        }

        // 3. update the grandpa state
        let next = if block.header.epoch_mark.is_some() {
            Some(self.storage.next_validators()?)
        } else {
            None
        };
        self.grandpa
            .write()
            .await
            .finalize(block.header.clone(), next)?;

        // 4. notify the new finalized block
        self.hook.on_finalized_block(block).await?;

        Ok(())
    }

    /// Validate a block header.
    #[tracing::instrument(skip_all, name = "importer::validate")]
    pub async fn validate(&self, header: &Header) -> anyhow::Result<()> {
        let handshake = self.grandpa.read().await.handshake.clone();
        let local_epoch = handshake.head.slot / score::EPOCH_LENGTH;
        let remote_epoch = header.slot / score::EPOCH_LENGTH;

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
        let slot = (header.slot % score::EPOCH_LENGTH) as usize;
        let entropy_buffer = self.storage.entropy()?;
        let mut ticket = None;
        let entropy = if header.epoch_mark.is_some() {
            entropy_buffer[2]
        } else {
            entropy_buffer[3]
        };

        // check the ticket mark
        if new_epoch {
            if let Ok(tickets) = self.storage.next_series() {
                ticket = Some(tickets[slot]);
            }
        } else if let Ok(TicketsOrKeys::Tickets(tickets)) = self.storage.series() {
            ticket = Some(tickets[slot]);
        }

        // indicate the keys to be used
        let keys = if new_epoch {
            self.storage.next_validators()?
        } else {
            self.storage.current_validators()?
        }
        .iter()
        .map(|v| v.bandersnatch)
        .collect::<Vec<_>>();

        // construct the message
        let mut oheader = header.clone();
        oheader.seal = [0; 96];
        oheader.entropy_source = [0; 96];
        let context = codec::encode(&oheader)?;

        // construct the context
        let mut message = Vec::new();
        if let Some(ticket) = ticket {
            message = TicketBody::message(ticket.attempt, &entropy);
        } else {
            message.extend_from_slice(&score::JAM_FALLBACK_SEAL);
            message.extend_from_slice(&entropy);
        }

        // check the ticket seal
        if let Some(ticket) = ticket {
            tracing::trace!(
                "verifying header seal with entropy: 0x{}, using ticket#{}@0x{}, author_index: {}",
                hex::encode(entropy.as_ref()),
                ticket.attempt,
                hex::encode(ticket.id),
                header.author_index
            );
        } else {
            tracing::trace!(
                "verifying header seal with entropy: 0x{},",
                hex::encode(entropy.as_ref())
            );
        }
        let author_index = header.author_index;
        let verifier = crypto::ring::verifier(keys.clone());
        let output = verifier
            .ietf_vrf_verify(&message, &context, &header.seal, author_index as usize)
            .map_err(|e| anyhow::anyhow!("ticket seal verification failed: {}", e))?;

        tracing::trace!("vrf verification output: 0x{}", hex::encode(output));
        if let Some(ticket) = ticket {
            if ticket.id != output {
                let TicketsOrKeys::Tickets(tickets) = self.storage.series()? else {
                    anyhow::bail!("ticket series not found");
                };
                tracing::error!(
                    "ticket series: {:?}",
                    tickets.into_iter().map(|t| t.id).collect::<Vec<_>>()
                );
                anyhow::bail!("header seal mismatched");
            }
        }

        // verify entropy source
        verifier
            .ietf_vrf_verify(
                &[],
                &[
                    &score::JAM_ENTROPY[..],
                    &crypto::vrf::ietf_output(header.seal)?[..],
                ]
                .concat(),
                &header.entropy_source,
                author_index as usize,
            )
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("entropy source verification failed: {}", e))?;

        Ok(())
    }
}
