//! Spacejam work package builder
//! Work package computation

pub use {
    bundle::WorkPackageBundle,
    segment::{InMemorySegmentProvider, SegmentBundle, SegmentProvider},
    worker::Worker,
};

mod bundle;
pub mod segment;
mod worker;
