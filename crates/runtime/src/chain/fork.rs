//! chain of blocks.

use crate::{
    chain::Grid,
    storage::{Branch, Commit, KVStorage, StateStorage},
    tx, Storage,
};
use anyhow::Result;
use pvm::Pvm;
use score::{
    block::{Head, Header},
    extrinsic::{TicketBody, TicketsOrKeys},
    Block, TimeSlot, TrieKey,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// A block with a diff.
pub type BlockWithDiff = (Block, Commit<TrieKey, Vec<u8>>);

/// A chain of blocks.
pub struct Fork<S: Storage> {
    /// The ancestors of the chain.
    pub chain: BTreeSet<Head>,

    /// The diff of the chain.
    pub blocks: BTreeMap<TimeSlot, BlockWithDiff>,

    /// The grid of the network.
    pub grid: Grid,

    /// The state of the chain.
    pub state: Branch<S>,

    /// tickets or keys for this fork chain per epoch.
    pub series: BTreeMap<u32, TicketsOrKeys>,
}

impl<S: Storage> Fork<S> {
    /// Create a new fork.
    pub fn new(state: Branch<S>, grid: Grid, series: BTreeMap<TimeSlot, TicketsOrKeys>) -> Self {
        Self {
            chain: BTreeSet::new(),
            blocks: BTreeMap::new(),
            grid,
            state,
            series,
        }
    }

    /// Get the best head of the chain.
    pub fn best(&self) -> Result<Head> {
        self.chain
            .iter()
            .last()
            .cloned()
            .ok_or(anyhow::anyhow!("best block not exists"))
    }

    /// Get the head of the chain.
    pub fn head(&self) -> Result<Head> {
        self.chain
            .iter()
            .next()
            .cloned()
            .ok_or(anyhow::anyhow!("head block not exists"))
    }

    /// Get the length of the chain.
    pub fn len(&self) -> usize {
        self.chain.len()
    }

    /// Check if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.chain.is_empty()
    }

    /// Create a fork of a fork
    pub fn fork<Vm: Pvm>(&self, block: &Block) -> Result<Self> {
        let parent = block.header.parent;
        let timeslot = self
            .chain
            .iter()
            .find(|h| h.hash == parent)
            .map(|h| h.slot)
            .ok_or(anyhow::anyhow!("parent block not found"))?;

        // checkout branch and commit diffs
        let branch = Branch::checkout(self.state.state());
        for (slot, (_, commit)) in self.blocks.iter() {
            if slot > &timeslot {
                break;
            }

            branch.commit_legacy(commit.clone())?;
        }

        // import the block
        let mut fork = Fork::new(branch, self.grid.clone(), self.series.clone());
        fork.import::<Vm>(block)?;
        Ok(fork)
    }

    /// Insert a new block to the chain.
    pub fn import<Vm: Pvm>(&mut self, block: &Block) -> Result<()> {
        let parent = self.best()?;

        // 1. check the parent
        if block.header.parent != parent.hash {
            anyhow::bail!(
                "invalid parent: 0x{} != 0x{}",
                hex::encode(block.header.parent[..3].as_ref()),
                hex::encode(parent.hash[..3].as_ref())
            );
        }

        // 2. check the state root
        let root = self.state.root()?;
        if block.header.parent_state_root != root {
            anyhow::bail!(
                "invalid parent state root: 0x{} != 0x{}",
                hex::encode(block.header.parent_state_root),
                hex::encode(root)
            );
        }

        // 3. verify the header
        self.validate(&block.header)?;

        // 4. transit the global state
        //
        // We execute the block instead of querying the latest state from the remote.
        let hash = block.header.hash()?;
        let diff = tx::transit::<Vm>(block.clone(), Arc::new(self.state.clone()))?;
        tracing::info!(
            "imported block#{}@{}, previous block#{}@{}",
            block.header.slot,
            hex::encode(&hash[..3]),
            parent.slot,
            hex::encode(parent.hash[..3].as_ref())
        );

        // 5. save the block and the diff
        self.blocks.insert(block.header.slot, (block.clone(), diff));

        // 6. update tickets or keys if any
        let Some(series) = block.header.tickets_mark else {
            return Ok(());
        };

        let epoch = block.header.slot / score::EPOCH_LENGTH + 1;
        tracing::info!(
            "tickets for epoch={epoch}: {:#?}",
            series
                .iter()
                .enumerate()
                .map(|(i, t)| format!("{:02}: 0x{}", i, hex::encode(t.id)))
                .collect::<Vec<_>>()
        );

        let series = TicketsOrKeys::Tickets(series);
        self.series.insert(epoch, series);
        Ok(())
    }

    /// Validate a block header.
    #[tracing::instrument(skip_all, name = "chain::validate")]
    pub fn validate(&self, header: &Header) -> anyhow::Result<()> {
        let best = self.best()?;
        let local_epoch = best.slot / score::EPOCH_LENGTH;
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
        let entropy_buffer = self.state.entropy()?;
        let mut ticket = None;
        let entropy = if new_epoch {
            entropy_buffer[2]
        } else {
            entropy_buffer[3]
        };

        // check the ticket mark
        if new_epoch {
            if let Some(TicketsOrKeys::Tickets(tickets)) = self.series.get(&remote_epoch) {
                ticket = Some(tickets[slot]);
            }
        } else if let Some(TicketsOrKeys::Tickets(tickets)) = self.series.get(&local_epoch) {
            ticket = Some(tickets[slot]);
        }

        // indicate the keys to be used
        let keys = if new_epoch {
            self.state.next_validators()?
        } else {
            self.state.current_validators()?
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
            .map_err(|e| {
                anyhow::anyhow!("ticket seal verification failed: {e}, new_epoch={new_epoch}")
            })?;

        if let Some(ticket) = ticket {
            if ticket.id != output {
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

impl<S: Storage> Clone for Fork<S> {
    fn clone(&self) -> Self {
        Self {
            chain: self.chain.clone(),
            blocks: self.blocks.clone(),
            grid: self.grid.clone(),
            state: self.state.clone(),
            series: self.series.clone(),
        }
    }
}
