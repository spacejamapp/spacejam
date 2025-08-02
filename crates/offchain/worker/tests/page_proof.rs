//! Tests for page-proof functionality

use score::Segment;
use worker::d3l::PageProof;
use worker::{DataLake, InMemoryDataLake};

/// Test basic page-proof generation and verification
#[tokio::test]
async fn test_page_proof_generation() {
    // Create test segments (smaller than 64 to test single page)
    let segments: Vec<Segment> = (0..10)
        .map(|i| {
            let mut segment = [0u8; 4104];
            segment[0] = i;
            segment
        })
        .collect();

    let segments_root = crypto::blake2b(
        &segments
            .iter()
            .flat_map(|s| crypto::blake2b(s))
            .collect::<Vec<_>>(),
    );

    // Generate page-proof
    let page_proof = PageProof::generate(&segments, 0, &segments_root).unwrap();

    // Verify the page-proof has correct structure
    assert_eq!(page_proof.page_index, 0);
    assert_eq!(page_proof.segment_count(), 10);
    assert_eq!(page_proof.segment_hashes.len(), 10);

    // Verify each segment against the page-proof
    for (index, segment) in segments.iter().enumerate() {
        assert!(page_proof.verify_segment(segment, index as u16).unwrap());
    }
}

/// Test page-proof integration with SegmentProvider
#[tokio::test]
async fn test_segment_provider_page_proof_integration() {
    let provider = InMemoryDataLake::default();
    let work_package_hash = [1u8; 32];

    // Create test segments
    let segments: Vec<Segment> = (0..5)
        .map(|i| {
            let mut segment = [0u8; score::SEGMENT_SIZE as usize];
            segment[0] = i + 10; // Different from first test
            segment
        })
        .collect();

    // Export segments (should automatically generate page-proofs)
    let (segments_root, _segment_chunks) = provider
        .export_segments(&segments, &work_package_hash)
        .await
        .unwrap();

    // Verify page-proof was stored
    let page_proof = provider.page_proof(&segments_root, 0).await.unwrap();
    assert!(page_proof.is_some());

    let page_proof = page_proof.unwrap();
    assert_eq!(page_proof.page_index, 0);
    assert_eq!(page_proof.segment_count(), 5);

    // Verify segments can be justified using the stored page-proof
    for (index, segment) in segments.iter().enumerate() {
        assert!(page_proof.verify_segment(segment, index as u16).unwrap());
    }
}

/// Test multiple pages (>64 segments)
#[tokio::test]
async fn test_multiple_page_proofs() {
    let provider = InMemoryDataLake::default();
    let work_package_hash = [2u8; 32];

    // Create test segments (more than 64 to test multiple pages)
    let segments: Vec<Segment> = (0..100)
        .map(|i| {
            let mut segment = [0u8; score::SEGMENT_SIZE as usize];
            segment[0] = (i % 256) as u8;
            segment[1] = (i / 256) as u8;
            segment
        })
        .collect();

    // Export segments
    let (segments_root, _segment_chunks) = provider
        .export_segments(&segments, &work_package_hash)
        .await
        .unwrap();

    // Should have 2 pages: page 0 (64 segments) and page 1 (36 segments)
    let page_0 = provider.page_proof(&segments_root, 0).await.unwrap();
    let page_1 = provider.page_proof(&segments_root, 1).await.unwrap();

    assert!(page_0.is_some());
    assert!(page_1.is_some());

    let page_0 = page_0.unwrap();
    let page_1 = page_1.unwrap();

    assert_eq!(page_0.segment_count(), 64);
    assert_eq!(page_1.segment_count(), 36);

    // Verify segments in each page (Gray Paper: 64 segments per page)
    for i in 0..64 {
        assert!(page_0.verify_segment(&segments[i], i as u16).unwrap());
    }

    for i in 0..(100 - 64) {
        assert!(page_1.verify_segment(&segments[64 + i], i as u16).unwrap());
    }
}

/// Test page-proof verification fails for wrong segments
#[tokio::test]
async fn test_page_proof_verification_failure() {
    let segments: Vec<Segment> = (0..3)
        .map(|i| {
            let mut segment = [0u8; score::SEGMENT_SIZE as usize];
            segment[0] = i;
            segment
        })
        .collect();

    let segments_root = crypto::blake2b(
        &segments
            .iter()
            .flat_map(|s| crypto::blake2b(s))
            .collect::<Vec<_>>(),
    );

    let page_proof = PageProof::generate(&segments, 0, &segments_root).unwrap();

    // Create a different segment
    let wrong_segment = [99u8; score::SEGMENT_SIZE as usize];

    // Verification should fail
    assert!(!page_proof.verify_segment(&wrong_segment, 0).unwrap());
}
