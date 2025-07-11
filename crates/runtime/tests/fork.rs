//! Test the fork behavior of the runtime.

/* use anyhow::Result;
use score::{block::Header, state::key, Block, EntropyBuffer};
use spacejam_runtime::{
    storage::{MemoryDb, StateStorage, SyncStorage},
    Runtime, Validator,
};
use std::collections::HashMap;

/// A chain of blocks.
pub struct Chain {
    runtime: Runtime<()>,
}

impl Chain {
    /// Create a new chain.
    pub async fn new() -> Result<Self> {
        let validator = crypto::ed25519::KeyPair::dev();
        let runtime = Runtime::new(validator, MemoryDb::default(), ());
        let mut state = HashMap::new();
        state.insert(key::ENTROPY, codec::encode(&EntropyBuffer::default())?);
        runtime
            .chain_mut()
            .await
            .import_genesis(Default::default(), &state)
            .await?;
        Ok(Self { runtime })
    }

    /// Author a new block.
    pub async fn author(&self, parent: &Header) -> Result<Block> {
        let mut header = parent.clone();
        header.slot += 1;
        header.parent = parent.hash()?;
        let chain = self.runtime.chain().await;
        header.parent_state_root = chain.state.root()?;
        Ok(Block {
            header,
            ..Default::default()
        })
    }

    /// Generate a chain of blocks.
    pub async fn chain(&self, blocks: usize) -> Result<Vec<Header>> {
        let mut headers = Vec::new();
        for _ in 1..blocks {
            let finalized = self.runtime.finalized().await;
            let parent = self.runtime.chain().await.state.header(&finalized.hash)?;
            let block = self.author(&parent).await?;
            let header = block.header.clone();
            headers.push(header.clone());
            let head = header.head()?;
            self.runtime.import(&block).await?;
            self.runtime.chain_mut().await.grandpa.handshake.head = head;
        }

        Ok(headers)
    }
}

#[tokio::test]
async fn test_non_fork() -> Result<()> {
    let chain = Chain::new().await?;
    for _ in 1..100 {
        let finalized = chain.runtime.finalized().await;
        let parent = chain.runtime.chain().await.state.header(&finalized.hash)?;
        let block = chain.author(&parent).await?;
        chain.runtime.import(&block).await?;
        chain.runtime.finalize().await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_checkout() -> Result<()> {
    let chain = Chain::new().await?;
    chain.chain(100).await?;
    assert_eq!(chain.runtime.finalized().await.slot, 100);
    Ok(())
}
 */
