//! Fuzz related commands

use clap::Parser;
use std::path::PathBuf;

/// The fuzz command
#[derive(Parser)]
pub enum Fuzz {
    /// Fuzz the local node
    Local {
        /// The path to the unix socket
        #[clap(default_value = "/tmp/jam_target.sock")]
        socket: PathBuf,
    },

    /// Fuzz the trace file
    Trace {
        /// The path to the trace folder
        traces: PathBuf,
    },
}
