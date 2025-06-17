//! Encode the data into erasure-coded shards async

use crate::Config;
use anyhow::Result;
use tokio::task::JoinSet;

/// Encoder for erasure-coded shards using systematic Reed-Solomon coding.
#[derive(Debug, Clone)]
pub struct Encoder {
    /// The configuration
    config: Config,
    /// The segment size
    segment: usize,
    /// The count of pieces
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
        let batch = self.batch(data);

        // Process batches in parallel
        let mut set = JoinSet::new();
        for (batch_idx, pieces) in batch.into_iter().enumerate() {
            let config = self.config;
            let segment = self.segment;

            set.spawn(async move {
                let result = tokio::task::spawn_blocking(move || {
                    Self::encode_piece(pieces.into_iter(), config, segment)
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

        // Collect results in correct order
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
    pub fn encode_sync(&mut self, data: Vec<u8>) -> Result<Vec<Vec<u8>>> {
        let pieces = self.batch(data).into_iter().flatten();
        let (original, recovery) = Self::encode_piece(pieces, self.config, self.segment)?;
        Ok(original.into_iter().chain(recovery).collect())
    }

    /// Extract batch of pieces from the data
    fn batch(&mut self, data: Vec<u8>) -> Vec<Vec<Vec<u8>>> {
        let data = self.pad(data);
        let size = self.config.batch(self.segment).max(1).min(self.pieces);
        let batches: Vec<Vec<Vec<u8>>> = (0..self.pieces)
            .map(|round| {
                let ptr = round * self.config.shard;
                let mut piece = Vec::with_capacity(self.config.piece());
                for i in 0..self.config.original {
                    let pos = ptr + i * self.segment;
                    piece.extend_from_slice(&data[pos..pos + self.config.shard]);
                }
                piece
            })
            .collect::<Vec<_>>()
            .chunks(size)
            .map(|chunk| chunk.to_vec())
            .collect();

        batches
    }

    /// Shared encoding logic that consumes the data chunks
    fn encode_piece(
        pieces: impl Iterator<Item = Vec<u8>>,
        config: Config,
        segment: usize,
    ) -> Result<(Vec<Vec<u8>>, Vec<Vec<u8>>)> {
        let mut original: Vec<Vec<u8>> = vec![Vec::with_capacity(segment); config.original];
        let mut recovery: Vec<Vec<u8>> = vec![Vec::with_capacity(segment); config.recovery];

        for piece in pieces {
            let mut encoder = config.encoder()?;
            for i in 0..config.original {
                let start = i * config.shard;
                let end = start + config.shard;
                let shard = &piece[start..end];
                encoder.add_original_shard(shard)?;
                original[i].extend_from_slice(shard);
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
