//! Testing utilities
#![cfg(test)]

use crate::{
    runtime::{
        storage::{BlockStorage, MemoryDb},
        Config, Validator,
    },
    safrole::ValidatorsData,
    OpaqueHash,
};
use tracing_subscriber::{fmt::Subscriber, EnvFilter};
pub use {network::Network, node::Node, validator::TestValidator};

mod network;
mod node;
mod validator;

/// The testing validators
pub const TEST_VALIDATORS: [[u8; 32]; crate::VALIDATORS_COUNT as usize] =
    [[0; 32], [1; 32], [2; 32], [3; 32], [4; 32], [5; 32]];

/// The testing config
pub struct TestConfig;

impl Config for TestConfig {
    type Storage = MemoryDb;
    type Validator = TestValidator;
}

/// The testing validators
pub fn validators() -> ValidatorsData {
    TEST_VALIDATORS
        .iter()
        .map(|v| TestValidator::from(*v).data())
        .collect()
}

/// Setup the tracing subscriber
pub fn setup_tracing() {
    Subscriber::builder()
        .without_time()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

#[tokio::test]
async fn genesis() {
    let node = Node::genesis(OpaqueHash::default(), validators())
        .await
        .expect("failed to create genesis node");
    let block = node.genesis;

    // 1. check the block is stored
    let hash = block.hash().unwrap();
    let sblock = node.runtime.storage.get_block(&hash).unwrap();
    assert_eq!(block, sblock);

    // 2.check the latest finalized head is recorded
    let finalized = node.runtime.grandpa.read().await.handshake.head.clone();
    assert_eq!(finalized.hash, hash);
    assert_eq!(finalized.slot, block.header.slot);

    // 3. check the grandpa is updated
    let grandpa = node.runtime.grandpa.read().await;
    assert!(grandpa.handshake.leaves.is_empty());
    assert_eq!(grandpa.handshake.head, finalized);
    assert_eq!(grandpa.grid.next.to_vec(), node.validators);

    // 4. check the ancestry is set up
    let ancestry = grandpa.ancestors(&hash, block.header.parent);
    assert!(ancestry.is_empty());
    assert!(grandpa.is_descendant_of(hash, block.header.parent));
}
