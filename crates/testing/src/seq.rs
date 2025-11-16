//! Sequential test vectors

use crate::traces::{self, TestInput, TestOutput};
use anyhow::Result;
use runtime::storage::{MemoryDb, StateStorage};
use score::TimeSlot;
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
    history: BTreeMap<TimeSlot, HashMap<Vec<u8>, Vec<u8>>>,
    init: bool,
}

impl Processor {
    /// Process a test
    pub async fn process(&mut self, test: Test) -> Result<()> {
        tracing::debug!("processing test: {}", test.name);
        let input = TestInput::from_json(&test.input)?;
        let output = TestOutput::from_json(&test.output)?;
        let slot = input.block.header.slot;
        if self.history.contains_key(&slot) {
            let mut lstate = Default::default();
            for (cslot, state) in self.history.iter() {
                if *cslot == slot {
                    self.memdb.reset(lstate);
                    break;
                }

                lstate = state.clone();
            }
        }

        if !self.init {
            for keyval in input.pre_state.keyvals.clone() {
                self.memdb
                    .state_set(keyval.key, keyval.value)
                    .expect("failed to set keyval");
            }
            self.init = true;
        }

        if std::env::var("SPACEVM").is_ok_and(|v| v == "true") {
            traces::run_single::<spacevm::Compiler>(self.memdb.clone(), input, output).await?;
        } else {
            traces::run_single::<spacevm::Interpreter>(self.memdb.clone(), input, output).await?;
        }
        self.history.insert(slot, self.memdb.deep_clone());
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
            init: false,
            history: BTreeMap::new(),
        }
    }
}
