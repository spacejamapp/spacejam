//! Merkle related utilities

#![cfg(feature = "mmr")]

mod binary;
pub mod mmr;
mod trie;
pub mod trie31;

pub use binary::MerkleTree;
pub use trie::merkle as trie;
pub use trie31::trie as trie31;
