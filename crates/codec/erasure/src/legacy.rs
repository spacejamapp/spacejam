//! Sync codec implementation

use crate::Config;
use anyhow::Result;
use std::collections::HashSet;

/// Encode the data into erasure-coded shards using Reed-Solomon coding.
pub fn encode(mut data: Vec<u8>, config: Config) -> Result<Vec<Vec<u8>>> {
    let mut length = data.len();
    let piece = config.piece();
    if !length.is_multiple_of(piece) {
        data.extend(vec![0; piece - (length % piece)]);
        length = data.len();
    }

    // calculate the size of data per original shard and encode data per piece
    let segment = config.segment(length);
    let mut original: Vec<Vec<u8>> = vec![Vec::with_capacity(segment); config.original];
    let mut recovery: Vec<Vec<u8>> = vec![Vec::with_capacity(segment); config.recovery];
    for round in 0..(length / piece) {
        let ptr = round * config.shard;
        let mut encoder = config.encoder()?;
        for i in 0..config.original {
            let pos = ptr + i * segment;
            let symbol = &data[pos..pos + config.shard];
            encoder.add_original_shard(symbol)?;
            original[i].extend_from_slice(symbol);
        }

        let encoded = encoder.encode()?;
        for (i, word) in encoded.recovery_iter().enumerate() {
            recovery[i].extend_from_slice(word);
        }
    }

    Ok(original.into_iter().chain(recovery).collect())
}

/// Decode the data from erasure-coded shards using systematic Reed-Solomon coding.
pub fn decode(data: Vec<(usize, Vec<u8>)>, config: Config) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Err(anyhow::anyhow!("No data to decode"));
    }

    let segment = data[0].1.len();
    let length = segment * config.original;
    let mut result = vec![0u8; length];

    // Pre-compute which original shards we have for O(1) lookup
    let originals: HashSet<usize> = data
        .iter()
        .filter_map(|(idx, _)| {
            if *idx < config.original {
                Some(*idx)
            } else {
                None
            }
        })
        .collect();

    // Process each word position (round)
    for round in 0..(segment / config.shard) {
        let ptr = round * config.shard;
        let mut decoder = config.decoder()?;
        for (index, chunk) in data.iter() {
            let word = &chunk[ptr..ptr + config.shard];
            if *index >= config.original {
                decoder.add_recovery_shard(*index - config.original, word)?;
            } else {
                decoder.add_original_shard(*index, word)?;
                let pos = ptr + *index * segment;
                result[pos..pos + config.shard].copy_from_slice(word);
            }
        }

        // Decode and add only the missing original shards
        let decoded = decoder.decode()?;
        for (i, word) in decoded.restored_original_iter() {
            if !originals.contains(&i) {
                let pos = ptr + i * segment;
                result[pos..pos + config.shard].copy_from_slice(word);
            }
        }
    }

    Ok(result)
}
