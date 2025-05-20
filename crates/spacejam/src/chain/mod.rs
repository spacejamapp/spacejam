//! Chain Configurations

pub use {
    config::Config,
    spec::{ParsedSpec, Spec},
};

mod config;
mod spec;

/// Chain id
pub enum ChainId {
    /// Development chain
    Dev,
    /// Polkadot chain
    Polkadot,
    /// Other chain
    Other(String),
}
