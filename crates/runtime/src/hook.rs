//! Hooks for the runtime

use anyhow::Result;
use score::{Block, block::Head};

/// Hooks for the runtime
pub trait Hook {
    /// Called when a new best block is imported
    fn on_best_block(&self, _block: Head) -> Result<()> {
        Ok(())
    }

    /// Called when a new finalized block is imported
    fn on_finalized_block(&self, _block: Block) -> impl Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }
}

impl Hook for () {}
