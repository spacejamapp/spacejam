//! Erasure encoding and decoding

use anyhow::Result;
pub use config::Config;
pub use encode::Encoder;

mod config;
mod decode;
mod encode;
pub mod sync;

/// Encode the data into erasure-coded shards using systematic Reed-Solomon coding.
pub async fn encode(data: Vec<u8>) -> Result<Vec<Vec<u8>>> {
    let mut encoder = Encoder::new(Config::default());
    encoder.encode(data).await
}

/// Encode the data into erasure-coded shards using systematic Reed-Solomon coding.
pub fn encode_sync(data: Vec<u8>) -> Result<Vec<Vec<u8>>> {
    sync::encode(data, Config::default())
}

/// Decode the data from erasure-coded shards using systematic Reed-Solomon coding.
pub async fn decode(data: Vec<(usize, Vec<u8>)>) -> Result<Vec<u8>> {
    decode::decode(data, Config::default()).await
}

/// Decode the data from erasure-coded shards using systematic Reed-Solomon coding.
pub fn decode_sync(data: Vec<(usize, Vec<u8>)>) -> Result<Vec<u8>> {
    sync::decode(data, Config::default())
}
