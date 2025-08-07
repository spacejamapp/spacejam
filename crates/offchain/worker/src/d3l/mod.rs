//! Segment operations for JAM Protocol data availability
//!
//! This module handles:
//! - Segment import/export operations
//! - Erasure coding for segments
//! - Segment provider abstraction
//! - Bundle creation and erasure root computation

// pub mod erasure;
pub mod bundle;
pub mod justification;
pub mod proof;
mod lake;
pub mod shard;

pub use {
    justification::{Justification, JustificationPath},
    proof::{BundleShardJustification, PageProof, SegmentShardJustification},
    lake::{DataLake, InMemoryDataLake},
    shard::Shard,
};
