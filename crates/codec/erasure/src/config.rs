//! Config of erasure coding

use anyhow::Result;
use reed_solomon::{ReedSolomonDecoder, ReedSolomonEncoder};

/// Erasure coding parameters
#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// The shard size in bytes
    pub shard: usize,
    /// The number of original shards
    pub original: usize,
    /// The total number of shards
    pub recovery: usize,
    /// (async) The memory allocated for the encoder and decoder in bytes
    pub memory: usize,
}

impl Config {
    /// (W_E) The basic size of erasure-coded pieces in octets
    pub const fn piece(&self) -> usize {
        self.shard * self.original
    }

    /// The total number of shards
    pub const fn total(&self) -> usize {
        self.original + self.recovery
    }

    /// The number of round batches
    pub const fn batch(&self, segment: usize) -> usize {
        self.memory / segment / self.total()
    }

    /// The size of the segment in bytes
    pub fn segment(&self, data: usize) -> usize {
        data / self.original
    }

    /// Create a new Reed-Solomon encoder
    pub fn encoder(&self) -> Result<ReedSolomonEncoder> {
        ReedSolomonEncoder::new(self.original, self.recovery, self.shard).map_err(Into::into)
    }

    /// Create a new reed-solomon decoder
    pub fn decoder(&self) -> Result<ReedSolomonDecoder> {
        ReedSolomonDecoder::new(self.original, self.recovery, self.shard).map_err(Into::into)
    }
}

// The tiny config that matches polkajam 0.6.5
impl Default for Config {
    fn default() -> Self {
        Self {
            shard: 2,
            original: 2,
            recovery: 4,
            memory: 4 * 1024 * 1024,
        }
    }
}
