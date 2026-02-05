//! Merkle proof related utilities

use crate::blake2b;
use std::collections::HashMap;

/// Parallel threshold
const PARALLEL_THRESHOLD: usize = 64;

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

/// Compute the root of a Merkle tree from leaves.
///
/// Implements the well-balanced binary merkle tree from the graypaper (eq. simplemerkleroot):
/// - M_B(v, H) = H(v[0]) when len(v) == 1
/// - M_B(v, H) = N(v, H) otherwise
///
/// Where N is the node function (eq. merklenode):
/// - N(v, H) = zerohash when len(v) == 0
/// - N(v, H) = v[0] when len(v) == 1
/// - N(v, H) = H("$node" || N(v[..ceil(len/2)], H) || N(v[ceil(len/2)..], H)) otherwise
pub fn hroot(leaves: Vec<Vec<u8>>, hash: fn(&[u8]) -> [u8; 32]) -> [u8; 32] {
    match leaves.len() {
        0 => [0u8; 32],
        1 => hash(&leaves[0]), // M_B hashes single element
        _ => {
            // Convert node result (which may be a blob or hash) to [u8; 32]
            let result = node(&leaves, hash);
            let mut root = [0u8; 32];
            root.copy_from_slice(&result);
            root
        }
    }
}

/// The node function N from graypaper eq. merklenode.
/// Recursively splits at ceil(len/2) for well-balanced tree structure.
fn node(v: &[Vec<u8>], hash: fn(&[u8]) -> [u8; 32]) -> Vec<u8> {
    match v.len() {
        0 => vec![0u8; 32],
        1 => v[0].clone(), // Return the blob itself, not hashed
        len => {
            let mid = len.div_ceil(2);
            let (left, right) = if len >= PARALLEL_THRESHOLD {
                rayon::join(|| node(&v[..mid], hash), || node(&v[mid..], hash))
            } else {
                (node(&v[..mid], hash), node(&v[mid..], hash))
            };
            hash(&[b"node", &left[..], &right[..]].concat()).to_vec()
        }
    }
}

/// Compute the Merkle tree.
///
/// @deprecated use `node` instead
pub fn tree(leaves: Vec<Vec<u8>>, hash: fn(&[u8]) -> [u8; 32]) -> Vec<Vec<Vec<u8>>> {
    if leaves.is_empty() {
        return vec![vec![vec![0u8; 32]]];
    }

    if leaves.len() == 1 {
        return vec![vec![hash(&leaves[0]).to_vec()]];
    }

    // pad leaves
    let mut tree = Vec::new();
    let mut current = leaves;

    // build layers until we reach the root.
    loop {
        let mut layer = Vec::new();
        for i in (0..current.len()).step_by(2) {
            let left = &current[i];
            if let Some(right) = current.get(i + 1) {
                layer.push(hash(&[b"node", left.as_slice(), right.as_slice()].concat()).to_vec());
            } else {
                layer.push(left.clone());
            }
        }

        tree.push(layer.clone());
        if layer.len() == 1 {
            break;
        }

        current = layer;
    }

    tree
}

/// Binary Merkle Tree
///
/// FIXME: fix this while implementing M2 again.
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
        if leaves.is_empty() {
            return Self {
                root: [0u8; 32],
                proofs: HashMap::new(),
            };
        }

        if leaves.len() == 1 {
            let root = blake2b(&leaves[0]);
            let mut proofs = HashMap::new();
            proofs.insert(
                leaves[0].clone(),
                MerkleProof {
                    root,
                    leaf: leaves[0].clone(),
                    index: 0,
                    proof: vec![],
                },
            );
            return Self { root, proofs };
        }

        // pad leaves
        let tree = tree(leaves.clone(), blake2b);
        let mut root = [0; 32];
        root.copy_from_slice(&tree[tree.len() - 1][0]);
        let padded_len = leaves.len().next_power_of_two();
        leaves.resize(padded_len, vec![]);

        // Generate proofs using the tree layers
        let mut proofs = HashMap::new();
        for (i, leaf) in leaves.iter().enumerate() {
            let mut proof_path = Vec::new();
            let mut index = i;

            // for leaf level, store sibling data as [u8; 32] (padded or hashed)
            let sibling_index = if index % 2 == 0 { index + 1 } else { index - 1 };
            let sibling_data = &leaves[sibling_index];
            let sibling_hash = if sibling_data.len() <= 31 {
                let mut padded = [0u8; 32];
                padded[0] = sibling_data.len() as u8;
                if !sibling_data.is_empty() {
                    padded[1..1 + sibling_data.len()].copy_from_slice(sibling_data);
                }
                padded
            } else {
                blake2b(sibling_data)
            };
            proof_path.push(sibling_hash);
            index /= 2;

            // for internal levels, find siblings in tree layers
            for layer in tree.iter().take(tree.len() - 1) {
                if layer.len() > 1 {
                    let sibling_index = if index % 2 == 0 { index + 1 } else { index - 1 };
                    if sibling_index < layer.len() {
                        let mut node = [0; 32];
                        node.copy_from_slice(&layer[sibling_index]);
                        proof_path.push(node);
                    }
                    index /= 2;
                }
            }

            proofs.insert(
                leaf.clone(),
                MerkleProof {
                    root,
                    leaf: leaf.clone(),
                    index: i as u16,
                    proof: proof_path,
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

        if self.proof.is_empty() {
            return self.root == blake2b(&self.leaf);
        }

        let mut proof = vec![self.leaf.clone()];
        for (i, h) in self.proof.iter().enumerate() {
            if i == 0 {
                let len = h[0] as usize;
                if len == 0 {
                    proof.push(vec![]);
                } else if len <= 31 {
                    proof.push(h[1..1 + len].to_vec());
                } else {
                    proof.push(h.to_vec());
                }
            } else {
                proof.push(h.to_vec());
            }
        }

        // do the verification using index to traverse up the tree
        let mut current = self.index;
        for _ in 0..self.proof.len() {
            let current_data = &proof[0];
            let sibling_data = &proof[1];

            let (left, right) = if current.is_multiple_of(2) {
                (current_data.as_slice(), sibling_data.as_slice())
            } else {
                (sibling_data.as_slice(), current_data.as_slice())
            };

            let parent_hash = blake2b(&[b"node", left, right].concat());
            proof.remove(0);
            proof[0] = parent_hash.to_vec();
            current /= 2;
        }

        // The remaining element should be the root
        if proof.len() != 1 {
            return false;
        }

        let computed_root = if proof[0].len() == 32 {
            let mut root = [0u8; 32];
            root.copy_from_slice(&proof[0]);
            root
        } else {
            blake2b(&proof[0])
        };

        self.root == computed_root
    }
}

#[ignore = "need to be fixed after removing padding"]
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
