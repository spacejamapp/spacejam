//! Testing utilities
#![cfg(test)]

use crate::{
    block::Block,
    extrinsic::TicketBody,
    runtime::{
        storage::{BlockStorage, MemoryDb},
        Config, Head, Runtime, Validator,
    },
    OpaqueHash,
};
use tracing_subscriber::{fmt::Subscriber, EnvFilter};
use validator::TestValidator;

mod validator;

pub struct TestConfig;

impl Config for TestConfig {
    type Storage = MemoryDb;
    type Validator = TestValidator;
}

const TEST_VALIDATORS: [[u8; 32]; crate::VALIDATORS_COUNT as usize] =
    [[0; 32], [1; 32], [2; 32], [3; 32], [4; 32], [5; 32]];

/// The testing node
pub struct Node {
    /// The runtime
    pub runtime: Runtime<TestConfig>,

    /// The genesis block
    pub genesis: Block,

    /// The validators
    pub validators: Vec<TestValidator>,
}

impl Node {
    /// Create a new testing node
    pub fn new(seed: OpaqueHash, genesis: Block, validators: Vec<TestValidator>) -> Self {
        let storage = MemoryDb::default();
        let validator = TestValidator::from(seed);
        let runtime = Runtime::new(validator, storage);
        Self {
            runtime,
            genesis,
            validators,
        }
    }

    /// Create a new testing node with the genesis block
    pub async fn genesis(seed: OpaqueHash) -> anyhow::Result<Self> {
        let validators: Vec<TestValidator> = TEST_VALIDATORS
            .iter()
            .map(|v| TestValidator::from(*v))
            .collect();

        let mut current = [[0; 32]; crate::VALIDATORS_COUNT as usize];
        for i in 0..crate::VALIDATORS_COUNT as usize {
            current[i] = validators[i].bandersnatch_public_key();
        }

        // create the genesis block
        let block = Block::genesis(current);
        let validators_data = validators.iter().map(|v| v.data()).collect::<Vec<_>>();

        // create the node
        let node = Self::new(seed, block.clone(), validators);
        node.runtime
            .import_genesis(&block, validators_data.as_slice())
            .await
            .expect("failed to import genesis block");
        Ok(node)
    }
}

fn setup_tracing() {
    Subscriber::builder()
        .without_time()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

#[tokio::test]
async fn genesis() {
    setup_tracing();

    let node = Node::genesis(OpaqueHash::default())
        .await
        .expect("failed to create genesis node");
    let block = node.genesis;

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
    assert_eq!(
        grandpa.grid.next.to_vec(),
        node.validators
            .iter()
            .map(|v| v.bandersnatch_public_key())
            .collect::<Vec<_>>()
    );

    // 4. check the ancestry is set up
    let ancestry = grandpa.ancestors(&hash, block.header.parent);
    assert!(ancestry.is_empty());
    assert!(grandpa.is_descendant_of(hash, block.header.parent));
}

#[tokio::test]
async fn author() {
    setup_tracing();

    // create the genesis block
    let node = Node::genesis(OpaqueHash::default())
        .await
        .expect("failed to create genesis node");
    let block = node.genesis;
    // 1. get the next block
    let (next, ticket) = node.runtime.next().await.expect("failed to get next block");
    assert_eq!(next.header.parent, block.header.hash().unwrap());

    // 2. verify the ticket
    //
    // NOTE: we don't always have a ticket since block authoring is slot based.
    if let Some(ticket) = ticket {
        assert_eq!(ticket.attempt, 0);

        // 2.1. verify the ticket signature
        let message = TicketBody::message(ticket.attempt, &[0; 32]);
        let keys = &node
            .validators
            .iter()
            .map(|v| v.bandersnatch_public_key())
            .collect::<Vec<_>>();
        let verifier = crypto::ring::verifier(keys.clone());
        verifier
            .ring_vrf_verify(&message, &[], &ticket.signature)
            .expect("failed to verify the ticket");
    }

    // 3. the block contains no ticket
    assert_eq!(next.extrinsic.tickets.len(), 1);
    assert!(next.header.tickets_mark.is_none());
}

#[ignore]
#[tokio::test]
async fn finalize() {
    setup_tracing();

    let node = Node::genesis(OpaqueHash::default())
        .await
        .expect("failed to create genesis node");

    // 1. author a block
    let (next, _) = node.runtime.next().await.expect("failed to get next block");

    // 2. finalize the block
    node.runtime
        .finalize(&next)
        .await
        .expect("failed to finalize block");

    // 3. check grandpa is updated
    let grandpa = node.runtime.grandpa.read().await;
    let head = Head {
        hash: next.hash().unwrap(),
        slot: next.header.slot,
    };
    assert_eq!(grandpa.handshake.head, head);
    assert!(grandpa.is_descendant_of(head.hash, node.genesis.header.parent));
}
