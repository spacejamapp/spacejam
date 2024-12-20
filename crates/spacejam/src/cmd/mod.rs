//! Command line interface for spacejam

use clap::Parser;
pub use rand::Rand;

mod rand;

/// The command line interface for spacejam
#[derive(Parser)]
pub enum Command {
    /// Generate random test data
    #[command(subcommand)]
    Rand(Rand),
}
