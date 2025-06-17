//! Sync codec implementation

use crate::Config;
use anyhow::Result;

/// Encode the data into erasure-coded shards using systematic Reed-Solomon coding.
pub fn encode(mut data: Vec<u8>, config: Config) -> Result<Vec<Vec<u8>>> {
    let mut length = data.len();
    let piece = config.piece();
    if length % piece != 0 {
        data.extend(vec![0; piece - (length % piece)]);
        length = data.len();
    }

    // Calculate the size of data per original shard and the number of encoding rounds
    let shard = length / config.original;
    let mut original: Vec<Vec<u8>> = vec![Vec::with_capacity(shard); config.original];
    let mut recovery: Vec<Vec<u8>> =
        vec![Vec::with_capacity(shard); config.total - config.original];

    // Process data in encoding rounds
    for round in 0..(length / piece) {
        let ptr = round * config.word;
        let mut encoder = config.encoder()?;
        for i in 0..config.original {
            let pos = ptr + i * shard;
            let symbol = &data[pos..pos + config.word];
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
