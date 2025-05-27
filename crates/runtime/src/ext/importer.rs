//! Importer for SpaceJam

use crate::{
    Config, Hook, Runtime, Storage,
    storage::{KVStorage, SyncStorage},
    tx,
};
use score::{
    Block,
    block::Header,
    extrinsic::{TicketBody, TicketsOrKeys},
    state::key,
};
use std::collections::HashMap;

impl<C: Config> Runtime<C> {
    /// Import the genesis block
    pub async fn import_genesis(
        &self,
        header: Header,
        state: &HashMap<[u8; 31], Vec<u8>>,
    ) -> anyhow::Result<()> {
        // 1. save the block to the storage
        let head = header.clone().try_into()?;
        let block = Block {
            header: header.clone(),
            ..Default::default()
        };
        self.storage.set_block(&block)?;
        self.storage.set_finalized(&head)?;

        // 2. set the genesis state
        let mut grandpa = self.grandpa.write().await;
        for (key, value) in state {
            self.storage.set(key, value)?;
            match *key {
                key::PREVIOUS_VALIDATORS => {
                    grandpa.grid.prev = codec::decode(value)?;
                }
                key::CURRENT_VALIDATORS => {
                    grandpa.grid.curr = codec::decode(value)?;
                }
                key::NEXT_VALIDATORS => {
                    grandpa.grid.next = codec::decode(value)?;
                }
                _ => {}
            }
        }

        grandpa.handshake.head = head;
        Ok(())
    }

    /// Initialize the runtime from the database
    pub async fn init_from_db(&self) -> anyhow::Result<()> {
        let finalized = self.storage.get_finalized()?;
        let mut grandpa = self.grandpa.write().await;
        grandpa.handshake.head = finalized;

        // apply validators
        let prev = self.storage.previous_validators().unwrap_or_default();
        let curr = self.storage.current_validators().unwrap_or_default();
        let next = self.storage.next_validators().unwrap_or_default();

        grandpa.grid.prev = prev;
        grandpa.grid.curr = curr;
        grandpa.grid.next = next;
        Ok(())
    }

    /// Finalize blocks
    ///
    /// Note that we only store finalized blocks and the blocks authored
    /// by ourselves in our storage.
    ///
    /// TODO: use block reference
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
        let diff = tx::transit::<C::Vm>(block.clone(), &self.storage)?;
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

        // 4. set the head as finalized
        self.storage
            .set_finalized(&block.header.clone().try_into()?)?;

        // 5. notify the new finalized block
        self.hook.on_finalized_block(block).await?;
        self.hook.on_diff(hash, diff).await?;
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
                "unhandled epoch: local: {}, remote: {}",
                local_epoch,
                remote_epoch
            );
        }

        // present the verifying components
        let new_epoch = remote_epoch > local_epoch;
        let slot = (header.slot % score::EPOCH_LENGTH) as usize;
        let entropy_buffer = self.storage.entropy()?;
        let mut ticket = None;
        let entropy = if new_epoch {
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
        let encoded = codec::encode(&header)?;
        let context = encoded[..encoded.len() - 96].to_vec();

        // construct the context
        let mut message = Vec::new();
        if let Some(ticket) = ticket {
            message = TicketBody::message(ticket.attempt, &entropy);
        } else {
            message.extend_from_slice(&score::JAM_FALLBACK_SEAL);
            message.extend_from_slice(&entropy);
        }

        // check the ticket seal
        let author_index = header.author_index;
        let verifier = crypto::ring::verifier(keys.clone());
        let output = verifier
            .ietf_vrf_verify(&message, &context, &header.seal, author_index as usize)
            .map_err(|e| anyhow::anyhow!("ticket seal verification failed: {}", e))?;

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
        let extracted_vrf_output = crypto::vrf::ietf_output(header.seal)?;
        let entropy_message = [&score::JAM_ENTROPY[..], &extracted_vrf_output[..]].concat();
        verifier
            .ietf_vrf_verify(
                &entropy_message,
                &[],
                &header.entropy_source,
                author_index as usize,
            )
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("entropy source verification failed: {}", e))?;

        Ok(())
    }
}
