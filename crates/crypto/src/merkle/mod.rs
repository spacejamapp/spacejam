//! Merkle related utilities

#![cfg(feature = "merkle")]

mod binary;
pub mod mmr;
mod trie;
pub mod trie31;

pub use binary::{broot, hroot, kroot, root, tree, MerkleTree};
pub use trie::merkle as trie;
pub use trie31::trie as trie31;
