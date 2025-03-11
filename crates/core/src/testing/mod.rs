//! Testing utilities
#![cfg(test)]

use crate::{
    block::Block,
    runtime::{
        storage::{BlockStorage, MemoryDb},
        Runtime,
    },
    safrole::ValidatorData,
    BandersnatchPublic, OpaqueHash,
};
use crypto::ed25519;
use tracing_subscriber::{fmt::Subscriber, EnvFilter};

/// The testing node
pub struct Node {
    /// The runtime
    pub runtime: Runtime<()>,
}

impl Node {
    /// Create a new testing node
    pub fn new(seed: OpaqueHash) -> Self {
        let storage = MemoryDb::default();
        let validator = ed25519::KeyPair::from(seed);
        let runtime = Runtime::new(validator, storage);
        Self { runtime }
    }

    /// Create a new testing node with the genesis block
    pub async fn genesis(block: Block, seed: OpaqueHash) -> anyhow::Result<Self> {
        let node = Self::new(seed);
        node.runtime
            .import_genesis(
                &block,
                &TEST_VALIDATORS
                    .iter()
                    .map(|v| ValidatorData {
                        bandersnatch: *v,
                        ed25519: *v,
                        bls: [0; 144],
                        metadata: [0; 128],
                    })
                    .collect::<Vec<_>>(),
            )
            .await
            .expect("failed to import genesis block");
        Ok(node)
    }
}

const TEST_VALIDATORS: [BandersnatchPublic; crate::VALIDATORS_COUNT as usize] =
    [[0; 32], [1; 32], [2; 32], [3; 32], [4; 32], [5; 32]];

fn setup_tracing() {
    Subscriber::builder()
        .without_time()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

#[tokio::test]
async fn genesis() {
    setup_tracing();

    let block = Block::genesis(TEST_VALIDATORS);
    let node = Node::genesis(block.clone(), OpaqueHash::default())
        .await
        .expect("failed to create genesis node");

    // 1. check the block is stored
    let hash = block.hash().unwrap();
    let sblock = node.runtime.storage.get_block(&hash).unwrap();
    assert_eq!(block, sblock);

    // 2.check the latest finalized head is recorded
    let finalized = node.runtime.storage.get_finalized().unwrap();
    assert_eq!(finalized.hash, hash);
    assert_eq!(finalized.slot, block.header.slot);

    // 3. check the grandpa is updated
    let grandpa = node.runtime.grandpa.read().await;
    assert!(grandpa.handshake.leaves.is_empty());
    assert_eq!(grandpa.handshake.head, finalized);
    assert_eq!(grandpa.grid.next, TEST_VALIDATORS);

    // 4. check the ancestry is set up
    let ancestry = grandpa.ancestors(&hash, block.header.parent);
    assert!(ancestry.is_empty());
    assert!(grandpa.is_descendant_of(hash, block.header.parent));
}

#[tokio::test]
async fn author() {
    setup_tracing();

    let block = Block::genesis(TEST_VALIDATORS);
    let node = Node::genesis(block.clone(), OpaqueHash::default())
        .await
        .expect("failed to create genesis node");

    // 1. get the next block
    let (next, _) = node.runtime.next().await.expect("failed to get next block");
    assert_eq!(next.header.parent, block.header.hash().unwrap());

    // 2. the block contains no ticket
    assert_eq!(next.extrinsic.tickets.len(), 1);
    assert!(next.header.tickets_mark.is_none());
}
