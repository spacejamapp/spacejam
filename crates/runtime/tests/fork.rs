//! Test the fork behavior of the runtime.

/* use anyhow::Result;
use score::{block::Header, Block};
use spacejam_runtime::{
    storage::{MemoryDb, SyncStorage},
    Runtime, Storage, Validator,
};

/// A chain of blocks.
pub struct Chain {
    runtime: Runtime<()>,
}

impl Chain {
    /// Create a new chain.
    pub async fn new() -> Result<Self> {
        let validator = crypto::ed25519::KeyPair::dev();
        let runtime = Runtime::new(validator, MemoryDb::default(), ());
        runtime
            .import_genesis(Default::default(), &Default::default())
            .await?;
        Ok(Self { runtime })
    }

    /// Author a new block.
    pub async fn author(&self, parent: &Header) -> Result<Block> {
        let mut header = parent.clone();
        header.slot += 1;
        header.parent = parent.hash()?;
        header.parent_state_root = self.runtime.storage.root()?;
        Ok(Block {
            header,
            ..Default::default()
        })
    }

    /// Generate a chain of blocks.
    pub async fn chain(&self, blocks: usize) -> Result<Vec<Header>> {
        let mut headers = Vec::new();
        for _ in 1..blocks {
            let finalized = self.runtime.storage.finalized()?;
            let parent = self.runtime.storage.header(&finalized.hash)?;
            let block = self.author(&parent).await?;
            let header = block.header.clone();
            headers.push(header.clone());
            let head = header.head()?;
            self.runtime.import(block.clone()).await?;
            self.runtime.storage.finalize(&head)?;
        }

        Ok(headers)
    }
}

#[tokio::test]
async fn test_non_fork() -> Result<()> {
    let chain = Chain::new().await?;
    for _ in 1..100 {
        let finalized = chain.runtime.storage.finalized()?;
        let parent = chain.runtime.storage.header(&finalized.hash)?;
        let block = chain.author(&parent).await?;
        let head = block.header.head()?;
        chain.runtime.import(block.clone()).await?;
        chain.runtime.storage.finalize(&head)?;
    }

    Ok(())
}

#[tokio::test]
async fn test_checkout() -> Result<()> {
    let chain = Chain::new().await?;
    chain.chain(100).await?;
    assert_eq!(chain.runtime.storage.finalized()?.slot, 100);
    Ok(())
}
 */
