//! Merkle proof related utilities

use crate::blake2b;
use std::collections::HashMap;

// Binary Merkle Tree with 16-bit `ChunkIndex` has depth at most 17.
// The proof has at most `depth - 1` length.
const MAX_MERKLE_PROOF_DEPTH: u32 = 16;

/// Compute the root of a Merkle tree from chunks using Blake2b.
pub fn broot(chunks: Vec<Vec<u8>>) -> [u8; 32] {
    root(chunks, blake2b)
}

/// Compute the root of a Merkle tree from chunks using Keccak.
pub fn kroot(chunks: Vec<Vec<u8>>) -> [u8; 32] {
    root(chunks, crate::keccak)
}

/// Compute the root of a Merkle tree from chunks.
pub fn root(leaves: Vec<Vec<u8>>, hash: fn(&[u8]) -> [u8; 32]) -> [u8; 32] {
    hroot(leaves, hash)
}

/// Compute the root of a Merkle tree from hashes.
pub fn hroot(hashes: Vec<Vec<u8>>, hash: fn(&[u8]) -> [u8; 32]) -> [u8; 32] {
    if hashes.is_empty() {
        return [0u8; 32];
    }

    let tree = tree(hashes.to_vec(), hash);
    tree[tree.len() - 1][0]
}

/// Compute the Merkle tree.
pub fn tree(leaves: Vec<Vec<u8>>, hash: fn(&[u8]) -> [u8; 32]) -> Vec<Vec<[u8; 32]>> {
    let depth = leaves.len().ilog2() as usize + 1;
    let mut tree = vec![vec![]; depth];
    tree[0] = leaves.iter().map(|leaf| hash(leaf.as_slice())).collect();

    for i in 1..depth {
        let len = 2usize.pow((depth - 1 - i) as u32);
        tree[i].resize(len, Default::default());
        let mut path = Vec::new();
        for j in 0..len {
            let prev = if i == 1 {
                leaves
                    .iter()
                    .map(|leaf| leaf.as_slice())
                    .collect::<Vec<&[u8]>>()
            } else {
                tree[i - 1]
                    .iter()
                    .map(|leaf| leaf.as_slice())
                    .collect::<Vec<&[u8]>>()
            };
            let left = prev[2 * j];
            let right = prev[2 * j + 1];
            path.push(hash(&[b"node", left, right].concat()));
        }
        tree[i] = path;
    }

    tree
}

/// Binary Merkle Tree
pub struct MerkleTree {
    root: [u8; 32],
    proofs: HashMap<Vec<u8>, MerkleProof>,
}

impl MerkleTree {
    /// Get the root of the tree.
    pub fn root(&self) -> [u8; 32] {
        self.root
    }

    /// Get the proof for a given leaf.
    pub fn proof(&self, leaf: &[u8]) -> Option<MerkleProof> {
        self.proofs.get(leaf).cloned()
    }
}

impl From<Vec<Vec<u8>>> for MerkleTree {
    fn from(mut leaves: Vec<Vec<u8>>) -> Self {
        leaves.resize(leaves.len().next_power_of_two(), Default::default());

        let tree = tree(leaves.clone(), blake2b);
        let mb_root = &tree[tree.len() - 1];
        assert_eq!(mb_root.len(), 1, "root must be a single hash");

        let root = mb_root[0];
        let depth = tree.len() - 1;
        let mut proofs = HashMap::new();
        for (i, leaf) in leaves.into_iter().enumerate() {
            let mut path = Vec::with_capacity(depth);
            let mut index = i;

            for layer in tree.iter().take(depth) {
                if index % 2 == 0 {
                    path.push(layer[index + 1]);
                } else {
                    path.push(layer[index - 1]);
                }
                index = index.div_ceil(2);
            }

            proofs.insert(
                leaf.clone(),
                MerkleProof {
                    root,
                    leaf,
                    index: i as u16,
                    proof: path,
                },
            );
        }

        Self { root, proofs }
    }
}

/// A proof for a leaf in a binary Merkle tree.
#[derive(Clone)]
pub struct MerkleProof {
    pub root: [u8; 32],
    pub leaf: Vec<u8>,
    pub index: u16,
    pub proof: Vec<[u8; 32]>,
}

impl MerkleProof {
    /// Verify the proof.
    pub fn verify(&self) -> bool {
        let index_bits = (MAX_MERKLE_PROOF_DEPTH - self.index.leading_zeros()) as usize;
        if index_bits > self.proof.len() {
            return false;
        }

        // TODO: zero copy
        let (mb_root, _) = self
            .proof
            .iter()
            .fold((self.leaf.clone(), 0), |(acc, i), hash| {
                let hash = hash.to_vec();
                let (left, right) = if get_bit(self.index, i) {
                    (hash, acc.clone())
                } else {
                    (acc.clone(), hash)
                };
                (
                    blake2b(&[b"node", left.as_slice(), right.as_slice()].concat()).to_vec(),
                    i + 1,
                )
            });

        if mb_root.len() != 32 {
            self.root == blake2b(&mb_root)
        } else {
            self.root.to_vec() == mb_root
        }
    }
}

/// Get the bit at the given index.
fn get_bit(bits: u16, i: usize) -> bool {
    bits & (1u16 << i) != 0
}

#[test]
fn verify_proof() {
    let verify = |chunks: Vec<Vec<u8>>| {
        let len = chunks.len();
        let tree = MerkleTree::from(chunks.clone());
        for (i, chunk) in chunks.into_iter().enumerate().take(len) {
            let proof = tree
                .proof(&chunk)
                .unwrap_or_else(|| panic!("Proof not found, chunks: {len}"));
            assert!(proof.verify(), "chunk index: {i}/{len}");
        }
    };

    verify(vec![
        vec![1, 2, 3, 4],
        vec![5, 6, 7, 8],
        vec![9, 10, 11, 12],
    ]);
    verify((0..255).map(|i| vec![i]).collect::<Vec<_>>());
}
