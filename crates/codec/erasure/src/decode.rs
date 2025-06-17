//! Decode the data from erasure-coded shards async

use crate::Config;
use anyhow::Result;
use std::{collections::HashSet, sync::Arc};
use tokio::task::JoinSet;

/// Async decode the data from erasure-coded shards using systematic Reed-Solomon coding.
pub async fn decode(data: Vec<(usize, Vec<u8>)>, config: Config) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Err(anyhow::anyhow!("No data to decode"));
    }

    let segment = data[0].1.len();
    let length = segment * config.original;
    let rounds = segment / config.shard;

    if rounds == 0 {
        return Ok(vec![0u8; length]);
    }

    // split rounds into batches
    let batches: Vec<Vec<usize>> = (0..rounds)
        .collect::<Vec<_>>()
        .chunks(config.batch(segment).max(1).min(rounds))
        .map(|chunk| chunk.to_vec())
        .collect();

    // process batches in parallel with order tracking
    let data = Arc::new(data);
    let mut set = JoinSet::new();
    for (batch_idx, batch) in batches.into_iter().enumerate() {
        let data = data.clone();
        set.spawn(async move {
            let result =
                tokio::task::spawn_blocking(move || process_batch(data, config, segment, batch))
                    .await
                    .map_err(|e| anyhow::anyhow!("Join error: {}", e))?;
            result.map(|batch_result| (batch_idx, batch_result))
        });
    }

    // process batches in parallel
    let mut results = set
        .join_all()
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
    results.sort_by_key(|(batch_idx, _)| *batch_idx);

    // collect results in correct order
    let mut final_result = vec![0u8; length];
    for (_batch_idx, batch_result) in results {
        for (pos, word) in batch_result {
            final_result[pos..pos + config.shard].copy_from_slice(&word);
        }
    }

    Ok(final_result)
}

/// Process a single batch of decode rounds (synchronous, CPU-intensive)
fn process_batch(
    data: Arc<Vec<(usize, Vec<u8>)>>,
    config: Config,
    segment: usize,
    rounds: Vec<usize>,
) -> Result<Vec<(usize, Vec<u8>)>> {
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

    let mut result = Vec::new();
    for round in rounds {
        let ptr = round * config.shard;
        let mut decoder = config.decoder()?;

        // Add shards to decoder and collect original shards
        for (index, chunk) in data.iter() {
            let word = &chunk[ptr..ptr + config.shard];
            if *index >= config.original {
                decoder.add_recovery_shard(*index - config.original, word)?;
            } else {
                decoder.add_original_shard(*index, word)?;
                let pos = ptr + *index * segment;
                result.push((pos, word.to_vec()));
            }
        }

        // Decode and add only the missing original shards
        let decoded = decoder.decode()?;
        for (i, word) in decoded.restored_original_iter() {
            if !originals.contains(&i) {
                let pos = ptr + i * segment;
                result.push((pos, word.to_vec()));
            }
        }
    }

    Ok(result)
}
