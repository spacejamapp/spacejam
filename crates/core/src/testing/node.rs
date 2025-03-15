//! Testing utilities
#![cfg(test)]

use std::ops::{Deref, DerefMut};

use crate::{
    block::Block,
    runtime::{storage::MemoryDb, Runtime},
    safrole::ValidatorsData,
    testing::{validator::TestValidator, TestConfig},
    OpaqueHash,
};

/// The testing node
pub struct Node {
    /// The runtime
    pub runtime: Runtime<TestConfig>,

    /// The genesis block
    pub genesis: Block,

    /// The validators
    pub validators: ValidatorsData,
}

impl Node {
    /// Create a new testing node
    pub fn new(seed: OpaqueHash, genesis: Block, validators: ValidatorsData) -> Self {
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
    pub async fn genesis(seed: OpaqueHash, validators: ValidatorsData) -> anyhow::Result<Self> {
        let current = validators
            .iter()
            .map(|v| v.bandersnatch)
            .collect::<Vec<_>>();

        // create the genesis block
        let block = Block::genesis(current.try_into().expect("failed to convert vec to array"));

        // create the node
        let node = Self::new(seed, block.clone(), validators);
        node.runtime
            .importer()
            .import_genesis(block, node.validators.as_slice())
            .await
            .expect("failed to import genesis block");
        Ok(node)
    }
}

impl Deref for Node {
    type Target = Runtime<TestConfig>;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}

impl DerefMut for Node {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.runtime
    }
}
