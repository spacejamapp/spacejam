//! Chain Configurations

use std::{fmt, str::FromStr};
pub use {
    config::Config,
    spec::{ParsedSpec, Spec},
};

mod config;
mod spec;

/// Chain id
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChainId {
    /// Development chain
    Dev,
    /// Polkadot chain
    Polkadot,
    /// Other chain
    Other(String),
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChainId::Dev => write!(f, "dev"),
            ChainId::Polkadot => write!(f, "polkadot"),
            ChainId::Other(s) => write!(f, "{}", s),
        }
    }
}

impl FromStr for ChainId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dev" => Ok(ChainId::Dev),
            "polkadot" => Ok(ChainId::Polkadot),
            _ => Ok(ChainId::Other(s.to_string())),
        }
    }
}
