//! The parity database storage

use anyhow::Result;
use parity_db::{
    BTreeIterator, ColumnOptions, Db, NewNode as PdNewNode, NodeRef as PdNodeRef, Operation as Op,
    Options,
};
use runtime::storage::{
    Column, Commit, KVStorage, MultiTree, NewNode, NodeAddress, NodeRef, Operation,
};
use score::{OpaqueHash, TrieKey};
use std::path::PathBuf;

const TRIE_COL: u8 = Column::TrieNodes as u8;

/// The parity database storage
pub struct Parity(Db);

impl KVStorage for Parity {
    fn commit(&self, column: Column, commit: Commit<TrieKey, Vec<u8>>) -> Result<()> {
        self.0.commit_changes(commit.ops().map(|op| match op {
            Operation::Set(k, v) => (column as u8, Op::Set(k.to_vec(), v)),
            Operation::Remove(k) => (column as u8, Op::Dereference(k.to_vec())),
        }))?;
        Ok(())
    }

    fn set(&self, column: Column, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()> {
        self.0.commit(vec![(
            column as u8,
            key.as_ref().to_vec(),
            Some(value.as_ref().to_vec()),
        )])?;
        Ok(())
    }

    fn get(&self, column: Column, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        Ok(self.0.get(column as u8, key.as_ref())?)
    }

    fn iter(&self, column: Column) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        Ok(ParityIter(self.0.iter(column as u8)?))
    }

    fn prefix_iter(
        &self,
        column: Column,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        let mut iter = self.0.iter(column as u8)?;
        iter.seek(prefix.as_ref())?;
        Ok(ParityIter(iter))
    }
}

impl MultiTree for Parity {
    fn insert_tree(&self, key: OpaqueHash, root: NewNode) -> Result<()> {
        self.0.commit_changes([(
            TRIE_COL,
            Op::InsertTree(key.to_vec(), to_pd_newnode(root)),
        )])?;
        Ok(())
    }

    fn dereference_tree(&self, key: OpaqueHash) -> Result<()> {
        self.0
            .commit_changes([(TRIE_COL, Op::DereferenceTree(key.to_vec()))])?;
        Ok(())
    }

    fn get_root(&self, key: OpaqueHash) -> Result<Option<(Vec<u8>, Vec<NodeAddress>)>> {
        Ok(self.0.get_root(TRIE_COL, key.as_ref())?)
    }

    fn get_node(&self, address: NodeAddress) -> Result<Option<(Vec<u8>, Vec<NodeAddress>)>> {
        Ok(self.0.get_node(TRIE_COL, address)?)
    }
}

fn to_pd_newnode(n: NewNode) -> PdNewNode {
    PdNewNode {
        data: n.data,
        children: n.children.into_iter().map(to_pd_noderef).collect(),
    }
}

fn to_pd_noderef(r: NodeRef) -> PdNodeRef {
    match r {
        NodeRef::New(n) => PdNodeRef::New(to_pd_newnode(n)),
        NodeRef::Existing(addr) => PdNodeRef::Existing(addr),
    }
}

/// The iterator wrapper
pub struct ParityIter<'a>(BTreeIterator<'a>);

impl Iterator for ParityIter<'_> {
    type Item = Result<(Vec<u8>, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map_err(Into::into).transpose()
    }
}

impl TryFrom<PathBuf> for Parity {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self> {
        let options = Options {
            path,
            columns: vec![
                // Column::State
                ColumnOptions {
                    btree_index: true,
                    ..Default::default()
                },
                // Column::Sync
                ColumnOptions {
                    btree_index: true,
                    ..Default::default()
                },
                // Column::Archive
                ColumnOptions {
                    btree_index: true,
                    ..Default::default()
                },
                // Column::TrieNodes (multitree, content-addressed trie nodes)
                ColumnOptions {
                    multitree: true,
                    preimage: true,
                    allow_direct_node_access: true,
                    ..Default::default()
                },
            ],
            sync_wal: true,
            sync_data: true,
            stats: true,
            salt: None,
            compression_threshold: Default::default(),
        };
        Ok(Parity(Db::open_or_create(&options)?))
    }
}
