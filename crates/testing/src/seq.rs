//! Sequential test vectors

use crate::traces::{self, TestInput, TestOutput};
use anyhow::Result;
use runtime::storage::{MemoryDb, StateStorage};
use specjam::Test;
use std::sync::Arc;

include!(concat!(env!("OUT_DIR"), "/traces_seq.rs"));

/// The processor for sequential test vectors
pub struct Processor {
    memdb: Arc<MemoryDb>,
    init: bool,
}

impl Processor {
    /// Create a new processor
    pub fn new() -> Self {
        Self {
            memdb: Arc::new(MemoryDb::default()),
            init: false,
        }
    }

    /// Process a test
    pub async fn process(&mut self, test: Test) -> Result<()> {
        let input = TestInput::from_json(&test.input)?;
        let output = TestOutput::from_json(&test.output)?;
        if !self.init {
            for keyval in input.pre_state.keyvals.clone() {
                self.memdb
                    .state_set(keyval.key, keyval.value)
                    .expect("failed to set keyval");
            }
            self.init = true;
        }

        traces::run_single(self.memdb.clone(), input, output).await?;
        Ok(())
    }
}
