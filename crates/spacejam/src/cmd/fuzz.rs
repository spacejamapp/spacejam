//! Fuzz related commands

use crate::fuzz::{self, fuzzer::Fuzzer, target::Target};
use clap::Parser;
use std::path::PathBuf;

/// The fuzz command
#[derive(Parser)]
pub enum Fuzz {
    /// Fuzz with local unix socket
    Local {
        /// The path to the unix socket
        #[clap(default_value = "/tmp/jam_target.sock")]
        socket: PathBuf,

        /// The path to the data folder
        #[clap(default_value = "spacejam_data", short, long)]
        data: PathBuf,
    },

    /// Fuzz with a fuzzer
    Fuzzer {
        /// The path to the fuzzer
        #[clap(default_value = "/tmp/jam_target.sock")]
        socket: PathBuf,

        /// The path to the traces folder
        #[clap(default_value = "jam-test-vectors/traces/storage", short, long)]
        traces: PathBuf,

        /// The path to the report folder
        #[clap(default_value = "reports", short, long)]
        report: PathBuf,
    },

    /// Run trace tests via the given trace folder
    Trace {
        /// The path to the trace folder
        traces: PathBuf,
    },
}

impl Fuzz {
    /// Run the fuzz command
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Local { socket, data } => Target::run(socket, data),
            Self::Fuzzer {
                socket,
                traces,
                report,
            } => Fuzzer::run(socket, traces, report),
            Self::Trace { traces } => fuzz::trace::test(traces),
        }
    }
}
