//! Sequential test vectors

use crate::traces::{self, TestInput, TestOutput};
use anyhow::Result;
use runtime::storage::{MemoryDb, StateStorage};
use score::{OpaqueHash, TimeSlot};
use specjam::Test;
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::Instant,
};

include!(concat!(env!("OUT_DIR"), "/traces_seq.rs"));

/// The processor for sequential test vectors
pub struct Processor {
    memdb: Arc<MemoryDb>,
    history: BTreeMap<OpaqueHash, HashMap<Vec<u8>, Vec<u8>>>,
}

impl Processor {
    /// Process a test
    pub async fn process(&mut self, test: Test) -> Result<()> {
        let input = TestInput::from_json(&test.input)?;
        let output = TestOutput::from_json(&test.output)?;
        let slot = input.block.header.slot;
        let hash = input.block.header.hash();
        if let Some(state) = self.history.get(&input.block.header.parent) {
            self.memdb.reset(state.clone());
        } else {
            self.memdb.reset(input.pre_state.keyvals());
        }

        let timeslot = self.memdb.timeslot()?;
        tracing::debug!(
            "processing test: {}, slot: {timeslot}, incoming: {slot}",
            test.name,
        );

        let is_ok = if std::env::var("SPACEVM").is_ok_and(|v| v == "true") {
            traces::run_single::<spacevm::Compiler>(self.memdb.clone(), input, output).await?
        } else {
            traces::run_single::<spacevm::Interpreter>(self.memdb.clone(), input, output).await?
        };

        if is_ok {
            self.history.insert(hash, self.memdb.deep_clone());
        }
        Ok(())
    }
}

impl Default for Processor {
    fn default() -> Self {
        let _ = tracing_subscriber::fmt::Subscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .without_time()
            .with_ansi(false)
            .with_thread_names(false)
            .with_file(false)
            // .with_level(false)
            .with_target(false)
            .try_init();

        Self {
            memdb: Arc::new(MemoryDb::default()),
            history: BTreeMap::new(),
        }
    }
}
