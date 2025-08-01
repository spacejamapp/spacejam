//! Spacejam work package builder
//! Work package computation

pub use {
    bundle::WorkPackageBundle,
    network::NetworkProvider,
    segment::{InMemorySegmentProvider, SegmentBundle, SegmentProvider},
    worker::Worker,
};

mod bundle;
mod network;
pub mod segment;
mod worker;
