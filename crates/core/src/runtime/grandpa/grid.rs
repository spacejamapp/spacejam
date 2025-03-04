//! The grid of the network.

use crate::{Ed25519Public, VALIDATORS_COUNT};
use std::collections::HashSet;

/// The grid of the network.
#[derive(Clone, Default)]
pub struct Grid {
    /// The previous layer of the grid.
    pub prev: [Ed25519Public; VALIDATORS_COUNT as usize],

    /// The current layer of the grid.
    pub curr: [Ed25519Public; VALIDATORS_COUNT as usize],

    /// The next layer of the grid.
    pub next: [Ed25519Public; VALIDATORS_COUNT as usize],
}

impl Grid {
    /// Get the neighbours of the given validator.
    ///
    /// Primarily for the purpose of block announcements, the previous, current, and next validator sets are conceptually arranged
    /// in a grid structure. Two validators are considered neighbours in the grid if:
    ///
    /// 1. They are validators in the same epoch and either have the same row (index / W) or the same column (index % W).
    ///     W here is floor(sqrt(V)), where V is the number of validators.
    /// 2. They are validators in different epochs but have the same index.
    pub fn neighbours(&self, validator: Ed25519Public) -> HashSet<Ed25519Public> {
        let layers = [&self.prev, &self.curr, &self.next];
        let mut neighbours = HashSet::new();

        // Check all three layers for the validator
        let (layer_index, index) = {
            let mut validator_layer = None;
            let mut validator_index = None;
            for (layer_index, layer) in layers.iter().enumerate() {
                if let Some(idx) = layer.iter().position(|v| *v == validator) {
                    validator_layer = Some(layer_index);
                    validator_index = Some(idx);
                    break;
                }
            }
            match (validator_layer, validator_index) {
                (Some(l), Some(i)) => (l, i),
                _ => return neighbours,
            }
        };

        // Calculate grid width and dimensions
        let width = (VALIDATORS_COUNT as f64).sqrt().floor() as usize;
        if width == 0 {
            return neighbours;
        }

        let row = index / width;
        let col = index % width;

        // Add same-row neighbors
        let layer = layers[layer_index];
        for (i, validator) in layer
            .iter()
            .enumerate()
            .take(((row + 1) * width).min(VALIDATORS_COUNT as usize))
            .skip(row * width)
        {
            if i != index {
                neighbours.insert(*validator);
            }
        }

        // Add same-column neighbors
        let mut col_idx = col;
        while col_idx < VALIDATORS_COUNT as usize {
            if col_idx != index {
                neighbours.insert(layer[col_idx]);
            }
            col_idx += width;
        }

        // Add cross-epoch neighbors (same index in different layers)
        if index < VALIDATORS_COUNT as usize {
            [0, 1, 2]
                .iter()
                .filter(|&i| *i != layer_index)
                .for_each(|i| {
                    neighbours.insert(layers[*i][index]);
                });
        }

        neighbours
    }
}

impl TryFrom<(Vec<Ed25519Public>, Vec<Ed25519Public>, Vec<Ed25519Public>)> for Grid {
    type Error = anyhow::Error;

    fn try_from(
        (prev, curr, next): (Vec<Ed25519Public>, Vec<Ed25519Public>, Vec<Ed25519Public>),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            prev: prev
                .try_into()
                .map_err(|_| anyhow::anyhow!("failed to convert prev"))?,
            curr: curr
                .try_into()
                .map_err(|_| anyhow::anyhow!("failed to convert curr"))?,
            next: next
                .try_into()
                .map_err(|_| anyhow::anyhow!("failed to convert next"))?,
        })
    }
}
