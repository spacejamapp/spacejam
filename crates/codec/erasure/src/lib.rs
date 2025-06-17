//! Erasure encoding and decoding

use anyhow::Result;
pub use config::Config;

mod config;
mod sync;

/// Encode the data into erasure-coded shards using systematic Reed-Solomon coding.
pub fn encode(data: Vec<u8>) -> Result<Vec<Vec<u8>>> {
    sync::encode(data, Config::default())
}
