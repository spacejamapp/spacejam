//! `MultiTree::apply` produces the same root as `trie31::trie`, both from
//! scratch and after an incremental diff against a prior tree.
#![cfg(feature = "merkle")]

use spacejam_crypto::{
    blake2b,
    merkle::{
        multitree::{MultiTree, MultiTreeMap},
        trie31,
    },
};

fn key(seed: u64) -> [u8; 31] {
    let mut k = [0u8; 31];
    k.copy_from_slice(&blake2b(&seed.to_le_bytes())[..31]);
    k
}

#[test]
fn from_scratch_matches_trie31() {
    let store = MultiTreeMap::default();
    let pairs: Vec<([u8; 31], Vec<u8>)> = (0..500u64)
        .map(|i| (key(i), i.to_le_bytes().repeat(5)))
        .collect();
    let kvs: Vec<([u8; 31], &[u8])> = pairs.iter().map(|(k, v)| (*k, v.as_slice())).collect();
    let dirty: Vec<[u8; 31]> = kvs.iter().map(|(k, _)| *k).collect();

    let root = store.apply(None, &kvs, &dirty).expect("apply");
    assert_eq!(root, trie31::trie(&kvs));
}

#[test]
fn incremental_matches_trie31() {
    let store = MultiTreeMap::default();
    let mut pairs: Vec<([u8; 31], Vec<u8>)> = (0..500u64)
        .map(|i| (key(i), i.to_le_bytes().repeat(5)))
        .collect();
    pairs.sort_by_key(|p| p.0);
    let kvs: Vec<([u8; 31], &[u8])> = pairs.iter().map(|(k, v)| (*k, v.as_slice())).collect();
    let dirty0: Vec<[u8; 31]> = kvs.iter().map(|(k, _)| *k).collect();

    let root0 = store.apply(None, &kvs, &dirty0).expect("round 0");
    assert_eq!(root0, trie31::trie(&kvs));

    let mut dirty1 = Vec::new();
    for i in 0..5u64 {
        let k = key(i);
        let v = (i + 1_000).to_le_bytes().repeat(7);
        if let Some(p) = pairs.iter_mut().find(|p| p.0 == k) {
            p.1 = v;
        }
        dirty1.push(k);
    }
    dirty1.sort();
    pairs.sort_by_key(|p| p.0);
    let kvs1: Vec<([u8; 31], &[u8])> = pairs.iter().map(|(k, v)| (*k, v.as_slice())).collect();

    let root1 = store.apply(Some(root0), &kvs1, &dirty1).expect("round 1");
    assert_eq!(root1, trie31::trie(&kvs1));
}
