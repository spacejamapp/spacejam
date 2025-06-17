//! Encode the data into erasure-coded shards async

use crate::Config;
use anyhow::Result;
use std::sync::Arc;
use tokio::task::JoinSet;

/// Encoder for erasure-coded shards using systematic Reed-Solomon coding.
#[derive(Debug, Clone)]
pub struct Encoder {
    /// The configuration
    config: Config,
    /// The segment size
    segment: usize,
    /// The number of rounds
    pieces: usize,
}

impl Encoder {
    /// Create a new encoder
    pub fn new(config: Config) -> Self {
        Self {
            config,
            segment: 0,
            pieces: 0,
        }
    }

    /// Async encode the data into erasure-coded shards using systematic Reed-Solomon coding.
    pub async fn encode(mut self, data: Vec<u8>) -> Result<Vec<Vec<u8>>> {
        let data = self.pad(data);
        if self.pieces == 0 {
            return Ok(vec![vec![]; self.config.original + self.config.recovery]);
        }

        // split rounds into batches
        let batches: Vec<Vec<usize>> = (0..self.pieces)
            .collect::<Vec<_>>()
            .chunks(self.config.batch(self.segment).max(1).min(self.pieces))
            .map(|chunk| chunk.to_vec())
            .collect();

        // Process batches in parallel using the same approach as the standalone encode function
        let data = Arc::new(data);
        let mut set = JoinSet::new();
        for (batch_idx, batch) in batches.into_iter().enumerate() {
            let data = data.clone();
            let config = self.config;
            let segment = self.segment;
            set.spawn(async move {
                let result = tokio::task::spawn_blocking(move || {
                    Self::encode_piece(&data, config, segment, batch)
                })
                .await
                .map_err(|e| anyhow::anyhow!("Join error: {}", e))?;
                result.map(|(original, recovery)| (batch_idx, original, recovery))
            });
        }

        // Process batches in parallel
        let mut results = set
            .join_all()
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;
        results.sort_by_key(|(batch_idx, _, _)| *batch_idx);

        // Collect results in correct order - write directly to self
        let mut original = vec![vec![]; self.config.original];
        let mut recovery = vec![vec![]; self.config.recovery];
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

    /// Synchronous encode the data into erasure-coded shards using systematic Reed-Solomon coding.
    ///
    /// TODO: consume the data on encoding, and returns iterator.
    pub fn encode_sync(&mut self, data: Vec<u8>) -> Result<Vec<Vec<u8>>> {
        let data = self.pad(data);
        if self.pieces == 0 {
            return Ok(vec![vec![]; self.config.original + self.config.recovery]);
        }

        let (original, recovery) =
            Self::encode_piece(&data, self.config, self.segment, (0..self.pieces).collect())?;
        Ok(original.into_iter().chain(recovery).collect())
    }

    /// Shared encoding logic that works with any data source
    fn encode_piece(
        data: &[u8],
        config: Config,
        segment: usize,
        rounds: Vec<usize>,
    ) -> Result<(Vec<Vec<u8>>, Vec<Vec<u8>>)> {
        let mut original: Vec<Vec<u8>> = vec![Vec::with_capacity(segment); config.original];
        let mut recovery: Vec<Vec<u8>> = vec![Vec::with_capacity(segment); config.recovery];

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

    /// Pad the data to the nearest multiple of the piece size
    fn pad(&mut self, mut data: Vec<u8>) -> Vec<u8> {
        let mut length = data.len();
        let piece = self.config.piece();
        if length % piece != 0 {
            data.extend(vec![0; piece - (length % piece)]);
            length = data.len();
        }

        self.segment = self.config.segment(length);
        self.pieces = length / self.config.piece();
        data
    }
}
