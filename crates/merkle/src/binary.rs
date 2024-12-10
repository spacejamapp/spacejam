//! Merkle proof related utilities

use crypto::blake2b;
use std::collections::HashMap;

/// Binary Merkle Tree
pub struct MerkleTree {
    root: [u8; 32],
    proofs: HashMap<[u8; 32], MerkleProof>,
}

impl MerkleTree {
    /// Get the root of the tree.
    pub fn root(&self) -> [u8; 32] {
        self.root
    }

    /// Get the proof for a given leaf.
    pub fn proof(&self, leaf: [u8; 32]) -> Option<MerkleProof> {
        self.proofs.get(&leaf).cloned()
    }
}

impl From<Vec<Vec<u8>>> for MerkleTree {
    fn from(chunks: Vec<Vec<u8>>) -> Self {
        let leaves = chunks
            .into_iter()
            .map(|chunk| blake2b(&chunk))
            .collect::<Vec<[u8; 32]>>();
        let tree = tree(leaves.clone());
        let root = tree[tree.len() - 1][0];

        let depth = tree.len() - 1;
        let mut proofs = HashMap::new();
        for (i, leaf) in leaves.into_iter().enumerate() {
            let mut path = Vec::new();
            let mut index = i;
            for j in 0..depth {
                if index % 2 == 0 {
                    path.push(tree[j][i + 1]);
                } else {
                    path.push(tree[j][i - 1]);
                }
                index /= 2;
            }

            proofs.insert(
                leaf,
                MerkleProof {
                    root,
                    leaf,
                    index: i,
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
    pub leaf: [u8; 32],
    pub index: usize,
    pub proof: Vec<[u8; 32]>,
}

impl MerkleProof {
    /// Verify the proof.
    pub fn verify(&self, root: [u8; 32]) -> bool {
        self.root == root
    }
}

fn tree(leaves: Vec<[u8; 32]>) -> Vec<Vec<[u8; 32]>> {
    let depth = leaves.len().ilog2() as usize + 1;
    let mut tree = Vec::new();
    tree.push(leaves);

    for i in 1..depth {
        let len = 2usize.pow((depth - 1 - i) as u32);
        let mut path = Vec::new();
        for j in 0..len {
            let prev = tree[i - 1].clone();
            let left = prev[2 * j];
            let right = prev[2 * j + 1];
            path.push(blake2b(&[left, right].concat()));
        }
        tree.push(path);
    }

    tree
}
