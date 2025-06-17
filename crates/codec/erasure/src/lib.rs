//! Erasure encoding and decoding

use anyhow::Result;
pub use config::Config;

mod config;
mod sync;

/// Encode the data into erasure-coded shards using systematic Reed-Solomon coding.
pub fn encode(data: Vec<u8>) -> Result<Vec<Vec<u8>>> {
    sync::encode(data, Config::default())
}

/// Decode the data from erasure-coded shards using systematic Reed-Solomon coding.
pub fn decode(data: Vec<(usize, Vec<u8>)>) -> Result<Vec<u8>> {
    sync::decode(data, Config::default())
}
