//! Spacejam testing library

use tracing_subscriber::EnvFilter;

/// Initialize tracing subscriber
pub fn init_tracing() {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

pub mod codec;
pub mod disputes;
pub mod history;
pub mod safrole;
pub mod shuffle;
pub mod statistics;
pub mod trie;
