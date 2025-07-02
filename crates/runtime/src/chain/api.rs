//! Chain APIs for the runtime.

use crate::{storage::StateStorage, Chain, Config, Grid, Handshake, Runtime};
use score::{
    block::{Head, Header},
    extrinsic::TicketsOrKeys,
    safrole::ValidatorIter,
    Block, EntropyBuffer,
};
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};

impl<C: Config> Runtime<C> {
    /// Add a leaf to the handshake.
    ///
    /// Returns `true` if the leaf is already in the chain.
    pub async fn add_leaf_to(
        &self,
        head: Head,
        leaf: &Header,
        handshake: &mut Handshake,
    ) -> anyhow::Result<bool> {
        let chain = self.chain().await;
        let mut exists = false;
        let mut added = false;
        for fork in chain.forks.values() {
            for block in fork.chain.iter() {
                if block.hash == leaf.parent {
                    handshake.add_leaf(fork.chain.clone(), head.clone());
                    added = true;
                }

                if block.hash == head.hash {
                    exists = true;
                }
            }

            if added {
                break;
            }
        }

        if !added {
            handshake.leaves.insert(head);
        }

        Ok(exists)
    }

    /// Get the best block of the runtime
    pub async fn best(&self) -> anyhow::Result<Head> {
        let chain = self.chain().await;
        if let Ok(fork) = chain.best_chain() {
            Ok(fork.best()?)
        } else {
            Ok(chain.grandpa.handshake.head.clone())
        }
    }

    /// Get the chain of the runtime
    pub async fn chain(&self) -> RwLockReadGuard<'_, Chain<C>> {
        self._chain.read().await
    }

    /// Get the chain of the runtime
    pub async fn chain_mut(&self) -> RwLockWriteGuard<'_, Chain<C>> {
        self._chain.write().await
    }

    /// Get the entropy of the runtime
    pub async fn entropy(&self) -> anyhow::Result<EntropyBuffer> {
        let chain = self.chain().await;
        if let Ok(fork) = chain.best_chain() {
            return fork.state.entropy();
        }

        chain.state.entropy()
    }

    /// Get the finalized block of the runtime
    pub async fn finalized(&self) -> Head {
        let chain = self.chain().await;
        chain.grandpa.handshake.head.clone()
    }

    /// Get the grid of the runtime
    pub async fn grid(&self) -> Grid {
        let chain = self.chain().await;
        if let Ok(fork) = chain.best_chain() {
            return fork.grid.clone();
        }

        chain.grid.clone()
    }

    /// Get the handshake of the runtime
    pub async fn handshake(&self) -> Handshake {
        let chain = self.chain().await;
        chain.grandpa.handshake.clone()
    }

    /// Import a block to the chain
    pub async fn import(&self, block: &Block) -> anyhow::Result<bool> {
        let mut chain = self.chain_mut().await;
        let imported = chain.import(block).await?;
        if imported {
            let head = block.header.head()?;
            let mut handshake = chain.grandpa.handshake.clone();
            drop(chain);
            tracing::trace!("adding leaf to the handshake");
            self.add_leaf_to(head, &block.header, &mut handshake)
                .await?;
            self.chain_mut().await.grandpa.handshake = handshake;
            tracing::trace!("leaf added");
        }

        Ok(imported)
    }

    /// Get the series for sealing / validating usages
    pub async fn series(&self, epoch: u32) -> anyhow::Result<TicketsOrKeys> {
        let chain = self.chain().await;
        if let Some(series) = chain.series.get(&epoch) {
            Ok(series.clone())
        } else if let Ok(fork) = chain.best_chain() {
            fork.series(epoch)
        } else {
            let validators = self.grid().await.next.bandersnatch();
            let entropy = self.entropy().await?;
            Ok(TicketsOrKeys::fallback(validators, entropy[1]))
        }
    }

    /// Get the tickets of the runtime
    pub async fn tickets(&self) -> u32 {
        let chain = self.chain().await;
        if let Ok(fork) = chain.best_chain() {
            fork.state.safrole().unwrap_or_default().accumulator.len() as u32
        } else {
            chain.state.safrole().unwrap_or_default().accumulator.len() as u32
        }
    }
}
