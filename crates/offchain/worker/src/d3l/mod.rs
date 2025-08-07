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
mod lake;
pub mod proof;
pub mod shard;

pub use {
    justification::{Justification, JustificationPath},
    lake::{DataLake, InMemoryDataLake},
    proof::{BundleShardJustification, PageProof, SegmentShardJustification},
    shard::Shard,
};
