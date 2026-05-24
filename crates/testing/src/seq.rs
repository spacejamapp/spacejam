//! Sequential test vectors

use crate::traces::{self, TestInput, TestOutput};
use anyhow::Result;
use runtime::tx::block::TestChain;
use specjam::Test;

include!(concat!(env!("OUT_DIR"), "/traces_seq.rs"));

/// The processor for sequential test vectors
pub struct Processor {
    chain: TestChain,
}

impl Processor {
    /// Process a test
    pub async fn process(&mut self, test: Test) -> Result<()> {
        let (input, output) = decode_trace(&test)?;
        if !self.chain.initialized() {
            self.chain.init(input.pre_state.keyvals())?;
        }

        let block = input.block.clone();
        let data = self.chain.prepare(&input.block);
        let is_ok = if std::env::var("SPACEVM").is_ok_and(|v| v == "true") {
            traces::run_single::<spacevm::SpaceVM, _>(data.clone(), input, output).await?
        } else {
            traces::run_single::<spacevm::Interpreter, _>(data.clone(), input, output).await?
        };

        if is_ok {
            self.chain.apply(&block, data)?;
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
            chain: TestChain::default(),
        }
    }
}

fn decode_trace(test: &Test) -> Result<(TestInput, TestOutput)> {
    if let Some(hex_data) = test.input.strip_prefix("bin:") {
        let bytes = hex::decode(hex_data)?;
        return traces::from_bin(&bytes);
    }
    let input = TestInput::from_json(&test.input)?;
    let output = TestOutput::from_json(&test.output)?;
    Ok((input, output))
}
