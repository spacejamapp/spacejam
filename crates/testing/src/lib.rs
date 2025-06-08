//! Spacejam testing library
#![cfg(test)]

use runner::Runner;
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
pub mod history;
pub mod preimage;
pub mod pvm;
pub mod reports;
pub mod runner;
pub mod safrole;
pub mod shuffle;
pub mod traces;
pub mod trie;

// FIXME: support parsing tuple from JSON
// pub mod statistics;
