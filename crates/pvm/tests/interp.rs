//! Entry point for the PVM tests.

use tracing_subscriber::EnvFilter;

mod format;

/// Initialize tracing subscriber.
pub fn init_tracing() {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}
