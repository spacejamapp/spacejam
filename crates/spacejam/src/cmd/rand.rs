//! Command `rand` for spacejam

use clap::Parser;

/// The `rand` command
///
/// Which generates random test data.
#[derive(Parser)]
pub enum Rand {
    /// Generate random genesis block in json format
    #[command(name = "genesis")]
    Genesis,
}
