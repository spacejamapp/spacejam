//! Spacejam work package builder
//! Work package computation

pub use {
    bundle::WorkPackageBundle,
    segment::{InMemorySegmentProvider, SegmentBundle, SegmentProvider},
    worker::Worker,
};

mod bundle;
mod segment;
mod worker;
