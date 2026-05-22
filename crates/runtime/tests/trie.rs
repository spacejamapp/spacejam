//! Cross-checks the incremental [`MultiTreeStore::apply`] root against the
//! reference [`crypto::merkle::trie31`] root.

use crypto::{blake2b, merkle};
use score::{OpaqueHash, TrieKey};
use spacejam_runtime::storage::{Column, MemoryDb, MultiTreeStore};
use std::collections::BTreeMap;

fn key(seed: u64) -> TrieKey {
    let mut k = [0u8; 31];
    let bytes = blake2b(&seed.to_le_bytes());
    k.copy_from_slice(&bytes[..31]);
    k
}

fn value(seed: u64, len: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(len);
    let mut s = seed;
    while v.len() < len {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        v.extend_from_slice(&s.to_le_bytes());
    }
    v.truncate(len);
    v
}

fn reference_root(state: &BTreeMap<TrieKey, Vec<u8>>) -> OpaqueHash {
    let kvs: Vec<(TrieKey, &[u8])> = state.iter().map(|(k, v)| (*k, v.as_slice())).collect();
    merkle::trie31(&kvs)
}

fn apply(
    db: &MemoryDb,
    prev: Option<OpaqueHash>,
    state: &BTreeMap<TrieKey, Vec<u8>>,
    dirty: &[TrieKey],
) -> OpaqueHash {
    let kvs: Vec<(TrieKey, &[u8])> = state.iter().map(|(k, v)| (*k, v.as_slice())).collect();
    db.apply(Column::TrieNodes, prev, &kvs, dirty)
        .expect("apply")
}

#[test]
fn empty_state() {
    let db = MemoryDb::default();
    let root = apply(&db, None, &BTreeMap::new(), &[]);
    assert_eq!(root, [0u8; 32]);
}

#[test]
fn single_leaf() {
    let db = MemoryDb::default();
    let mut state = BTreeMap::new();
    state.insert(key(1), value(1, 10));
    let dirty: Vec<_> = state.keys().copied().collect();
    let root = apply(&db, None, &state, &dirty);
    assert_eq!(root, reference_root(&state));
}

#[test]
fn bootstrap_matches_trie31() {
    let db = MemoryDb::default();
    let mut state = BTreeMap::new();
    for i in 0..500u64 {
        state.insert(key(i), value(i, 40));
    }
    let dirty: Vec<_> = state.keys().copied().collect();
    let root = apply(&db, None, &state, &dirty);
    assert_eq!(root, reference_root(&state));
}

#[test]
fn incremental_update_matches_trie31() {
    let db = MemoryDb::default();
    let mut state = BTreeMap::new();
    for i in 0..500u64 {
        state.insert(key(i), value(i, 40));
    }
    let dirty0: Vec<_> = state.keys().copied().collect();
    let root0 = apply(&db, None, &state, &dirty0);
    assert_eq!(root0, reference_root(&state));

    let mut dirty = Vec::new();
    for i in 0..5u64 {
        let k = key(i);
        state.insert(k, value(i + 1000, 50));
        dirty.push(k);
    }
    for i in 500..503u64 {
        let k = key(i);
        state.insert(k, value(i, 60));
        dirty.push(k);
    }
    for i in 100..102u64 {
        let k = key(i);
        state.remove(&k);
        dirty.push(k);
    }
    dirty.sort();

    let root1 = apply(&db, Some(root0), &state, &dirty);
    assert_eq!(root1, reference_root(&state));
}

#[test]
fn many_sequential_diffs_match_trie31() {
    let db = MemoryDb::default();
    let mut state = BTreeMap::new();
    for i in 0..200u64 {
        state.insert(key(i), value(i, 32));
    }
    let dirty: Vec<_> = state.keys().copied().collect();
    let mut root = apply(&db, None, &state, &dirty);
    assert_eq!(root, reference_root(&state));

    for round in 1..=20u64 {
        let mut dirty = Vec::new();
        for j in 0..10u64 {
            let k = key((round * 7 + j) % 200);
            // Offset by 1_000_000 so we never accidentally write back a
            // bootstrap value and leave the root unchanged.
            state.insert(k, value(round * 100 + j + 1_000_000, 32));
            dirty.push(k);
        }
        dirty.sort();
        dirty.dedup();
        root = apply(&db, Some(root), &state, &dirty);
        assert_eq!(root, reference_root(&state), "round {round}");
    }
}
