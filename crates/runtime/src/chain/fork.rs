//! chain of blocks.

use crate::{
    chain::Grid,
    storage::{Branch, Column, Commit, KVStorage, StateStorage},
    tx, Storage,
};
use anyhow::Result;
use pvm::Pvm;
use score::{
    block::{Head, Header},
    extrinsic::{TicketBody, TicketsOrKeys},
    safrole::ValidatorIter,
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
    pub state: Arc<Branch<S>>,

    /// tickets or keys for this fork chain per epoch.
    pub series: BTreeMap<u32, TicketsOrKeys>,
}

impl<S: Storage> Fork<S> {
    /// Create a new fork.
    pub fn new(
        state: Arc<Branch<S>>,
        grid: Grid,
        series: BTreeMap<TimeSlot, TicketsOrKeys>,
    ) -> Self {
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
    ///
    /// FIXME: there could be problem in this implementation.
    pub fn fork<Vm: Pvm>(&self, parent: &Head, block: &Block) -> Result<Self> {
        let timeslot = parent.slot;

        // checkout branch and commit diffs
        let branch = Branch::checkout(self.state.state());
        let mut blocks = BTreeMap::new();
        let mut chain = BTreeSet::new();
        for (slot, (this, commit)) in self.blocks.iter() {
            if slot > &timeslot {
                break;
            }

            chain.insert(this.header.head()?);
            blocks.insert(*slot, (this.clone(), commit.clone()));
            branch.commit(Column::State, commit.clone())?;
        }

        // import the block
        let mut fork = Fork {
            chain,
            blocks,
            grid: self.grid.clone(),
            state: Arc::new(branch),
            series: self.series.clone(),
        };
        fork.import::<Vm>(parent, block)?;
        Ok(fork)
    }

    /// Insert a new block to the chain.
    pub fn import<Vm: Pvm>(&mut self, parent: &Head, block: &Block) -> Result<()> {
        // 1. check the state root
        tracing::trace!("checking state root");
        let root = self.state.root()?;
        if block.header.parent_state_root != root {
            tracing::error!(
                "invalid parent state root: 0x{} != 0x{}",
                hex::encode(block.header.parent_state_root),
                hex::encode(root)
            );

            self.on_state_root_mismatch(block.clone(), block.header.parent_state_root, root)?;
            panic!(
                "if we meet this case, either we have problem in our branch or we got attacked."
            );
        }

        // 2. verify the header
        tracing::trace!("validating block header");
        self.validate(parent, &block.header)?;

        // 3. transit the global state
        //
        // We execute the block instead of querying the latest state from the remote.
        tracing::trace!("transiting block");
        let head = block.header.head()?;
        let diff = tx::simulate::<Vm>(&mut block.clone(), self.state.clone())?;
        self.state.commit(Column::State, diff.clone())?;
        tracing::info!(
            "imported block#{}@{}, previous block#{}@{}",
            block.header.slot,
            hex::encode(&head.hash[..3]),
            parent.slot,
            hex::encode(parent.hash[..3].as_ref())
        );

        // 4. save the block and the diff
        self.chain.insert(head);
        self.blocks.insert(block.header.slot, (block.clone(), diff));

        // 5. update fallback tickets if need
        let epoch = block.header.slot / score::EPOCH_LENGTH;
        let prev_epoch = parent.slot / score::EPOCH_LENGTH;
        if epoch > prev_epoch && !self.series.contains_key(&epoch) {
            let validators = self.state.next_validators()?.bandersnatch();
            let entropy = self.state.entropy()?;
            let series = TicketsOrKeys::fallback(validators, entropy[1]);
            self.series.insert(epoch, series);
        }

        // 6. update safrole tickets or keys if any
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

    /// Get the series for sealing / validating usages
    pub fn series(&self, epoch: u32) -> anyhow::Result<TicketsOrKeys> {
        if let Some(series) = self.series.get(&epoch) {
            Ok(series.clone())
        } else {
            let validators = self.state.next_validators()?.bandersnatch();
            let entropy = self.state.entropy()?;
            let series = TicketsOrKeys::fallback(validators, entropy[1]);
            Ok(series)
        }
    }

    /// Validate a block header.
    #[tracing::instrument(skip_all, name = "chain::validate")]
    pub fn validate(&self, parent: &Head, header: &Header) -> anyhow::Result<()> {
        let local_epoch = parent.slot / score::EPOCH_LENGTH;
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
            if let Ok(TicketsOrKeys::Tickets(tickets)) = self.series(remote_epoch) {
                ticket = Some(tickets[slot]);
            }
        } else if let Ok(TicketsOrKeys::Tickets(tickets)) = self.series(local_epoch) {
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
