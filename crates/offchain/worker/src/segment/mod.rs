//! Segment operations for JAM Protocol data availability
//!
//! This module handles:
//! - Segment import/export operations
//! - Erasure coding for segments
//! - Segment provider abstraction
//! - Bundle creation and erasure root computation

mod bundle;
pub mod justification;
mod provider;
pub mod shard;

pub use {
    bundle::SegmentBundle,
    justification::{BundleShardJustification, Justification, SegmentShardJustification},
    provider::{InMemorySegmentProvider, SegmentProvider},
};
