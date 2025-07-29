//! Segment operations for JAM Protocol data availability
//!
//! This module handles:
//! - Segment import/export operations
//! - Erasure coding for segments
//! - Segment provider abstraction
//! - Bundle creation and erasure root computation

mod bundle;
mod provider;
mod storage;

pub use {
    bundle::SegmentBundle,
    provider::{InMemorySegmentProvider, SegmentProvider},
    storage::SegmentStorage,
};
