//! Erasure encoding and decoding

use anyhow::Result;
use std::collections::HashMap;

/// Encode data with parity
pub fn encode(data: Vec<Vec<u8>>, count: usize) -> Result<Vec<Vec<u8>>> {
    let original = data.to_vec();

    reed_solomon::encode(count, count, original)
        .map_err(|e| anyhow::anyhow!("Failed to encode: {}", e))
}

/// Decode data with parity
pub fn decode(data: Vec<Vec<u8>>, count: usize) -> Result<HashMap<usize, Vec<u8>>> {
    reed_solomon::decode(
        count,
        count,
        [(0, []); 0],
        data.into_iter()
            .enumerate()
            .collect::<Vec<(usize, Vec<u8>)>>(),
    )
    .map_err(|e| anyhow::anyhow!("Failed to decode: {}", e))
}
