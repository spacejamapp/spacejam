//! Erasure encoding and decoding

use anyhow::Result;
pub use config::Config;
pub use decode::Decoder;
pub use encode::Encoder;

mod config;
mod decode;
mod encode;
pub mod legacy;

/// Encode the data into erasure-coded shards using systematic Reed-Solomon coding.
pub async fn encode(data: Vec<u8>) -> Result<Vec<Vec<u8>>> {
    Encoder::new(Config::default()).encode(data).await
}

/// Encode the data into erasure-coded shards using systematic Reed-Solomon coding.
pub fn encode_sync(data: Vec<u8>) -> Result<Vec<Vec<u8>>> {
    Encoder::new(Config::default()).encode_sync(data)
}

/// Decode the data from erasure-coded shards using systematic Reed-Solomon coding.
pub async fn decode(data: Vec<(usize, Vec<u8>)>) -> Result<Vec<u8>> {
    Decoder::new(Config::default()).decode(data).await
}

/// Decode the data from erasure-coded shards using systematic Reed-Solomon coding.
pub fn decode_sync(data: Vec<(usize, Vec<u8>)>) -> Result<Vec<u8>> {
    Decoder::new(Config::default()).decode_sync(data)
}
