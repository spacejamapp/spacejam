//! Config of erasure coding

use anyhow::Result;
use reed_solomon::ReedSolomonEncoder;

/// Erasure coding parameters
pub struct Config {
    /// The size of the word in bytes
    pub word: usize,
    /// The number of original shards
    pub original: usize,
    /// The total number of shards
    pub total: usize,
}

impl Config {
    /// (W_E) The basic size of erasure-coded pieces in octets
    pub const fn piece(&self) -> usize {
        self.word * self.original
    }

    /// Create a new Reed-Solomon encoder
    pub fn encoder(&self) -> Result<ReedSolomonEncoder> {
        ReedSolomonEncoder::new(self.original, self.total - self.original, self.word)
            .map_err(Into::into)
    }
}

// The tiny config that matches polkajam 0.6.5
impl Default for Config {
    fn default() -> Self {
        Self {
            word: 2,
            original: 2,
            total: 6,
        }
    }
}
