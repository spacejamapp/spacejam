//! Merkle related utilities

mod binary;
pub mod mmr;
mod trie;

pub use binary::MerkleTree;
pub use trie::merkle as trie;
