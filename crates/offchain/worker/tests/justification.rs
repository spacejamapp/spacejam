//! Justification tests

use worker::d3l::proof::{
    BundleShardJustification, Justification, JustificationPath, SegmentShardJustification,
};

#[test]
fn test_justification_path_single_node() {
    let root = [1u8; 32];
    let shard_hash = [1u8; 32];
    let path = JustificationPath::new(root, 0, vec![]);

    assert!(path.verify_shard(&shard_hash).unwrap());
}

#[test]
fn test_justification_path_with_siblings() {
    let shard_hash = [1u8; 32];
    let sibling_hash = [2u8; 32];

    // Compute expected root
    let expected_root = crypto::blake2b(&[&shard_hash[..], &sibling_hash[..]].concat());

    let path = JustificationPath::new(
        expected_root,
        0, // Left child
        vec![Justification::Hash(sibling_hash)],
    );

    assert!(path.verify_shard(&shard_hash).unwrap());
}

#[test]
fn test_justification_types() {
    let hash = [1u8; 32];
    let hash_pair = ([1u8; 32], [2u8; 32]);
    let shard_data = vec![1, 2, 3, 4];

    let j1 = Justification::Hash(hash);
    let j2 = Justification::HashPair(hash_pair.0, hash_pair.1);
    let j3 = Justification::SegmentShard(shard_data);

    assert_eq!(j1, Justification::Hash(hash));
    assert_eq!(j2, Justification::HashPair(hash_pair.0, hash_pair.1));
    assert_eq!(j3, Justification::SegmentShard(vec![1, 2, 3, 4]));
}

#[test]
fn test_justification_constructors() {
    // Create test shards
    let segment = vec![1u8; 4104];
    let shards = erasure::encode_sync(segment).unwrap();
    let merkle_tree = crypto::merkle::MerkleTree::from(shards.clone());
    let erasure_root = merkle_tree.root();

    // Test BundleShardJustification::new
    let bundle_justification = BundleShardJustification::new(&shards, &erasure_root, 0).unwrap();
    assert!(bundle_justification.is_some());
    let justification = bundle_justification.unwrap();
    assert_eq!(justification.shard_index, 0);

    // Test SegmentShardJustification::new
    let segment_justification =
        SegmentShardJustification::new(&shards, &erasure_root, 42, 0).unwrap();
    assert!(segment_justification.is_some());
    let justification = segment_justification.unwrap();
    assert_eq!(justification.segment_index, 42);
    assert_eq!(justification.shard_index, 0);

    // Test with invalid erasure root
    let wrong_root = [0u8; 32];
    let invalid_justification = BundleShardJustification::new(&shards, &wrong_root, 0).unwrap();
    assert!(invalid_justification.is_none());
}
