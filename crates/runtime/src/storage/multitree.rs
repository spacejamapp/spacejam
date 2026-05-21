//! Multi-tree (ref-counted Merkle node) storage abstraction.
//!
//! Mirrors parity-db's multitree API so the runtime can drive incremental trie
//! updates against either parity-db or the in-memory shim in [`MemoryDb`],
//! without depending on parity-db directly.

use crate::storage::Column;
use anyhow::{Result, anyhow};
use crypto::merkle::trie31;
use score::{OpaqueHash, TrieKey};

const ZERO_HASH: OpaqueHash = [0u8; 32];

/// Address of a node already persisted in the store.
pub type NodeAddress = u64;

/// A reference used when building a tree to commit: either a brand-new subtree
/// that must be written, or a pointer to an existing node that the new tree
/// should reuse (and which the store will ref-count automatically).
#[derive(Debug, Clone)]
pub enum NodeRef {
    /// A fresh subtree.
    New(NewNode),
    /// A node already in the store, retained as-is.
    Existing(NodeAddress),
}

/// A newly-built node with its child references.
#[derive(Debug, Clone)]
pub struct NewNode {
    /// Encoded node payload (leaf or branch).
    pub data: Vec<u8>,
    /// Child references, in canonical order.
    pub children: Vec<NodeRef>,
}

/// Ref-counted Merkle tree storage.
pub trait MultiTreeStore: Send + Sync + 'static {
    /// Commit a new tree rooted at `key`. Nodes referenced via
    /// [`NodeRef::Existing`] have their refcount bumped automatically.
    fn insert_tree(&self, column: Column, key: OpaqueHash, root: NewNode) -> Result<()>;

    /// Drop the tree at `key`. Nodes whose refcount falls to zero are GC'd.
    fn dereference_tree(&self, column: Column, key: OpaqueHash) -> Result<()>;

    /// Get the root node (encoded data + immediate children) for the tree at
    /// `key`, if present.
    fn get_root(
        &self,
        column: Column,
        key: OpaqueHash,
    ) -> Result<Option<(Vec<u8>, Vec<NodeAddress>)>>;

    /// Get a node by its address.
    fn get_node(
        &self,
        column: Column,
        address: NodeAddress,
    ) -> Result<Option<(Vec<u8>, Vec<NodeAddress>)>>;

    /// Compute the new state root from `new_state`, reusing unchanged subtrees
    /// of the tree at `prev_root`. Writes the new tree into the store and
    /// dereferences the old one if it differs.
    ///
    /// - The tree shape matches `crypto::merkle::trie31` (graypaper D.6), so
    ///   the returned root is bit-identical to the non-incremental reference.
    /// - `new_state` is the full sorted post-diff state.
    /// - `dirty_keys` is the sorted set of keys touched by the diff (inserts,
    ///   updates, removals). Subtrees whose dirty slice is empty are reused
    ///   via [`NodeRef::Existing`].
    fn apply(
        &self,
        column: Column,
        prev_root: Option<OpaqueHash>,
        new_state: &[(TrieKey, &[u8])],
        dirty_keys: &[TrieKey],
    ) -> Result<OpaqueHash> {
        let prev_root_node = match prev_root {
            Some(h) if h != ZERO_HASH => self.get_root(column, h)?,
            _ => None,
        };

        let (root_ref, new_root) = self.build(column, prev_root_node, new_state, dirty_keys, 0)?;

        // If the tree didn't change there's nothing to write or release; the
        // existing tree at `new_root` already holds every node the rebuild
        // would have referenced.
        if matches!(prev_root, Some(p) if p == new_root) {
            return Ok(new_root);
        }

        if let Some(NodeRef::New(root_node)) = root_ref {
            self.insert_tree(column, new_root, root_node)?;
        }

        if let Some(prev) = prev_root {
            if prev != ZERO_HASH && prev != new_root {
                self.dereference_tree(column, prev)?;
            }
        }

        Ok(new_root)
    }

    /// Build the subtree at this position, with `prev_node` being the matching
    /// node from the previous tree (if any). Internal helper for [`apply`].
    fn build(
        &self,
        column: Column,
        prev_node: Option<(Vec<u8>, Vec<NodeAddress>)>,
        new_keys: &[(TrieKey, &[u8])],
        new_dirty: &[TrieKey],
        depth: usize,
    ) -> Result<(Option<NodeRef>, OpaqueHash)> {
        if new_keys.is_empty() {
            return Ok((None, ZERO_HASH));
        }

        if new_keys.len() == 1 {
            let (k, v) = new_keys[0];
            let data = trie31::leaf(k, v).to_vec();
            let hash = crypto::blake2b(&data);
            return Ok((Some(NodeRef::New(NewNode { data, children: vec![] })), hash));
        }

        // Branch case: partition both keys and dirty by bit at `depth`.
        let mut buf = new_keys.to_vec();
        let key_mid = trie31::partition(&mut buf, depth);
        let (left_keys, right_keys) = buf.split_at(key_mid);
        let dirty_mid = trie31::split_at_bit(new_dirty, depth);
        let (left_dirty, right_dirty) = new_dirty.split_at(dirty_mid);

        let (prev_left, prev_right) = match &prev_node {
            Some((data, children)) if !trie31::is_leaf(data) => trie31::split_branch_children(
                data, children,
            )
            .ok_or_else(|| {
                anyhow!(
                    "trie node shape mismatch at depth {depth}: children.len()={}",
                    children.len()
                )
            })?,
            _ => (None, None),
        };

        let (l_ref, l_hash) = self.descend(column, prev_left, left_keys, left_dirty, depth + 1)?;
        let (r_ref, r_hash) = self.descend(column, prev_right, right_keys, right_dirty, depth + 1)?;

        let data = trie31::branch(l_hash, r_hash).to_vec();
        let hash = crypto::blake2b(&data);
        let mut children = Vec::with_capacity(2);
        if let Some(l) = l_ref {
            children.push(l);
        }
        if let Some(r) = r_ref {
            children.push(r);
        }
        Ok((Some(NodeRef::New(NewNode { data, children })), hash))
    }

    /// Recurse into a child of the current branch. Reuses the subtree as-is
    /// when no dirty key touches it. Internal helper for [`apply`].
    fn descend(
        &self,
        column: Column,
        prev_addr: Option<NodeAddress>,
        new_keys: &[(TrieKey, &[u8])],
        new_dirty: &[TrieKey],
        depth: usize,
    ) -> Result<(Option<NodeRef>, OpaqueHash)> {
        if new_dirty.is_empty() {
            if let Some(addr) = prev_addr {
                let (data, _children) = self
                    .get_node(column, addr)?
                    .ok_or_else(|| anyhow!("missing prev node at address {addr}"))?;
                let hash = crypto::blake2b(&data);
                return Ok((Some(NodeRef::Existing(addr)), hash));
            }
            if new_keys.is_empty() {
                return Ok((None, ZERO_HASH));
            }
            // Fall through: build fresh when caller's dirty tracking is
            // over-conservative.
        }

        let prev_node = match prev_addr {
            Some(addr) => self.get_node(column, addr)?,
            None => None,
        };
        self.build(column, prev_node, new_keys, new_dirty, depth)
    }
}
