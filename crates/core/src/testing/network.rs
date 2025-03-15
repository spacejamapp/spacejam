//! Testing network in memory

use crate::{
    runtime::Validator,
    safrole::ValidatorsData,
    testing::{self, Node, TEST_VALIDATORS},
    Ed25519Public,
};
use anyhow::Result;
use std::collections::HashMap;

/// The testing network
pub struct Network {
    /// The nodes
    pub nodes: HashMap<Ed25519Public, Node>,

    /// The validators
    pub validators: ValidatorsData,
}

impl Network {
    /// Initialize a new network
    pub async fn init() -> Result<Self> {
        testing::setup_tracing();
        let mut nodes = HashMap::new();
        let validators = testing::validators();
        for seed in TEST_VALIDATORS {
            let node = Node::genesis(seed, validators.clone()).await?;
            nodes.insert(node.runtime.validator.ed25519_public_key(), node);
        }
        Ok(Self { nodes, validators })
    }

    /// Run to the next slot
    pub async fn next(&mut self, timeslot: u32) -> Result<()> {
        for node in self.nodes.values_mut() {
            let mut author = node.author();
            let (block, ticket) = author.on_timeslot(timeslot).await?;

            // TODO: subscribe blocks and tickets to network
        }
        Ok(())
    }
}
