//! Decode the data from erasure-coded shards async

use crate::Config;
use anyhow::Result;
use std::collections::HashSet;
use tokio::task::JoinSet;

/// Decoder for erasure-coded shards using systematic Reed-Solomon coding.
#[derive(Debug, Clone)]
pub struct Decoder {
    /// The configuration
    config: Config,
    /// The segment size
    segment: usize,
    /// The count of pieces
    pieces: usize,
}

impl Decoder {
    /// Create a new decoder
    pub fn new(config: Config) -> Self {
        Self {
            config,
            segment: 0,
            pieces: 0,
        }
    }

    /// Async decode the data from erasure-coded shards using systematic Reed-Solomon coding.
    pub async fn decode(mut self, data: Vec<(usize, Vec<u8>)>) -> Result<Vec<u8>> {
        let batches = self.batch(data);

        // Process batches in parallel
        let mut set = JoinSet::new();
        for (batch_idx, (rounds, batch_data)) in batches.into_iter().enumerate() {
            let config = self.config;
            let segment = self.segment;
            set.spawn(async move {
                let result = tokio::task::spawn_blocking(move || {
                    Self::decode_piece(batch_data, config, segment, rounds)
                })
                .await
                .map_err(|e| anyhow::anyhow!("Join error: {}", e))?;
                result.map(|batch_result| (batch_idx, batch_result))
            });
        }

        // Process batches in parallel
        let mut results = set
            .join_all()
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;
        results.sort_by_key(|(batch_idx, _)| *batch_idx);

        // Collect results in correct order
        let length = self.segment * self.config.original;
        let mut final_result = vec![0u8; length];
        for (_batch_idx, batch_result) in results {
            for (pos, word) in batch_result {
                final_result[pos..pos + self.config.shard].copy_from_slice(&word);
            }
        }

        Ok(final_result)
    }

    /// Synchronous decode the data from erasure-coded shards using systematic Reed-Solomon coding.
    pub fn decode_sync(&mut self, data: Vec<(usize, Vec<u8>)>) -> Result<Vec<u8>> {
        let batches = self.batch(data);
        let length = self.segment * self.config.original;
        let mut final_result = vec![0u8; length];

        for (rounds, batch_data) in batches {
            let batch_result = Self::decode_piece(batch_data, self.config, self.segment, rounds)?;
            for (pos, word) in batch_result {
                final_result[pos..pos + self.config.shard].copy_from_slice(&word);
            }
        }

        Ok(final_result)
    }

    /// Create batches of round indices from the data (consuming the data efficiently)
    fn batch(&mut self, data: Vec<(usize, Vec<u8>)>) -> Vec<(Vec<usize>, Vec<(usize, Vec<u8>)>)> {
        if data.is_empty() {
            return vec![];
        }

        self.segment = data[0].1.len();
        self.pieces = self.segment / self.config.shard;
        if self.pieces == 0 {
            return vec![];
        }

        let size = self.config.batch(self.segment).max(1).min(self.pieces);
        (0..self.pieces)
            .collect::<Vec<_>>()
            .chunks(size)
            .map(|chunk| (chunk.to_vec(), data.clone()))
            .collect()
    }

    /// Shared decoding logic for processing a batch of data
    fn decode_piece(
        batch_data: Vec<(usize, Vec<u8>)>,
        config: Config,
        segment: usize,
        rounds: Vec<usize>,
    ) -> Result<Vec<(usize, Vec<u8>)>> {
        // Pre-compute which original shards we have for O(1) lookup
        let originals: HashSet<usize> = batch_data
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
            for (index, chunk) in &batch_data {
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
}
