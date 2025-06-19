//! Config for the JAM index

use clap::Parser;
use std::{net::SocketAddr, path::PathBuf};

/// Config for the JAM index
#[derive(Parser)]
pub struct Config {
    /// The postgres database url
    #[clap(long, default_value = "postgres://postgres@postgres")]
    pub postgres: String,

    /// The path to the chain data
    #[clap(long)]
    pub data: PathBuf,

    /// The path to the spec file
    #[clap(long)]
    pub spec: Option<PathBuf>,

    /// The graphql server address
    #[clap(long, default_value = "0.0.0.0:8080")]
    pub graphql: SocketAddr,

    /// The address for the quic transport
    #[clap(long, default_value = "0.0.0.0:0")]
    pub quic: SocketAddr,
}
