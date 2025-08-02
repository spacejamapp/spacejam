//! Shard tests

use score::SEGMENT_SIZE;
use worker::d3l::shard;

#[test]
fn test_round_trip() {
    let original = vec![1u8; SEGMENT_SIZE as usize];
    let shards = erasure::encode_sync(original.to_vec()).unwrap();
    let partial = shard::partial_shards(&shards);
    let reconstructed = shard::reconstruct_segment(&partial).unwrap();
    assert_eq!(original, reconstructed);
}

#[test]
fn test_verify_shard() {
    let mut segment = vec![0u8; SEGMENT_SIZE as usize];
    // Add some pattern to make shards different
    for i in 0..SEGMENT_SIZE as usize {
        segment[i] = (i % 256) as u8;
    }

    let shards = erasure::encode_sync(segment.to_vec()).unwrap();
    assert!(shard::verify_shard(&segment, &shards[0], 0).unwrap());
    assert!(shard::verify_shard(&segment, &shards[1], 1).unwrap());
    assert!(!shard::verify_shard(&segment, &shards[1], 0).unwrap());
}

#[test]
fn test_verify_root() {
    let segment = vec![1u8; SEGMENT_SIZE as usize];
    let shards = erasure::encode_sync(segment.to_vec()).unwrap();
    let merkle_tree = crypto::merkle::MerkleTree::from(shards.clone());
    let root = merkle_tree.root();
    assert!(shard::verify_root(&shards, &root).unwrap());

    let wrong_root = [0u8; 32];
    assert!(!shard::verify_root(&shards, &wrong_root).unwrap());
}
