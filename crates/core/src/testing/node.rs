//! Testing utilities
#![cfg(test)]

use super::validator::TestValidator;
use crate::{
    block::Block,
    extrinsic::TicketEnvelope,
    runtime::{storage::MemoryDb, Config, Runtime, Validator},
    OpaqueHash, TimeSlot,
};
use tracing_subscriber::{fmt::Subscriber, EnvFilter};

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
        self::setup_tracing();
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

    /// Author a block with a given timeslot
    pub async fn author(
        &self,
        timeslot: TimeSlot,
    ) -> anyhow::Result<(Block, Option<TicketEnvelope>)> {
        let ticket = self.runtime.ticket()?;
        if let Some(ticket) = ticket.clone() {
            self.runtime
                .expool
                .tickets
                .lock()
                .await
                .insert((self.validators[0].bandersnatch_public_key(), ticket));
        }

        Ok((self.runtime.author(timeslot).await?, ticket))
    }
}

fn setup_tracing() {
    Subscriber::builder()
        .without_time()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}
