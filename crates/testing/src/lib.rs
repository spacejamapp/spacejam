//! Spacejam testing library

#![allow(unused_imports)]

pub use runner::Runner;
pub use specjam::{Entry, Section, Test, Trace};
use tracing_subscriber::EnvFilter;

/// Initialize tracing subscriber
pub fn init_tracing() {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

pub mod accumulate;
pub mod assurances;
pub mod authorizations;
pub mod codec;
pub mod disputes;
pub mod erasure;
pub mod history;
pub mod preimage;
pub mod pvm;
pub mod reports;
pub mod runner;
pub mod safrole;
pub mod shuffle;
pub mod statistics;
pub mod traces;
pub mod trie;
