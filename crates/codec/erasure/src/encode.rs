//! Encode the data into erasure-coded shards async

use crate::Config;
use anyhow::Result;
use std::sync::Arc;
use tokio::task::JoinSet;

/// Async encode the data into erasure-coded shards using Reed-Solomon coding.
pub async fn encode(mut data: Vec<u8>, config: Config) -> Result<Vec<Vec<u8>>> {
    let mut length = data.len();
    let piece = config.piece();
    if length % piece != 0 {
        data.extend(vec![0; piece - (length % piece)]);
        length = data.len();
    }

    let segment = config.segment(length);
    let rounds = length / piece;
    if rounds == 0 {
        return Ok(vec![vec![]; config.original + config.recovery]);
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
            let result = tokio::task::spawn_blocking(move || {
                process_batch_sync(data, config, segment, batch)
            })
            .await
            .map_err(|e| anyhow::anyhow!("Join error: {}", e))?;
            result.map(|(original, recovery)| (batch_idx, original, recovery))
        });
    }

    // process batches in parallel
    let mut results = set
        .join_all()
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
    results.sort_by_key(|(batch_idx, _, _)| *batch_idx);

    // collect results in correct order
    let mut original: Vec<Vec<u8>> = vec![Vec::new(); config.original];
    let mut recovery: Vec<Vec<u8>> = vec![Vec::new(); config.recovery];
    for (_batch_idx, batch_original, batch_recovery) in results {
        for (i, shard) in batch_original.into_iter().enumerate() {
            original[i].extend(shard);
        }
        for (i, shard) in batch_recovery.into_iter().enumerate() {
            recovery[i].extend(shard);
        }
    }

    Ok(original.into_iter().chain(recovery).collect())
}

/// Process a single batch of rounds (synchronous, CPU-intensive)
fn process_batch_sync(
    data: Arc<Vec<u8>>,
    config: Config,
    segment: usize,
    rounds: Vec<usize>,
) -> Result<(Vec<Vec<u8>>, Vec<Vec<u8>>)> {
    let mut original: Vec<Vec<u8>> = vec![Vec::new(); config.original];
    let mut recovery: Vec<Vec<u8>> = vec![Vec::new(); config.recovery];

    for round in rounds {
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

    Ok((original, recovery))
}
