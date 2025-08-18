//! Fuzz related commands

use crate::fuzz::{self, fuzzer::Fuzzer, target::Target};
use clap::Parser;
use std::path::PathBuf;

/// The fuzz command
#[derive(Parser)]
pub enum Fuzz {
    /// Fuzz with local unix socket
    Target {
        /// The path to the unix socket
        #[clap(default_value = "/tmp/jam_target.sock")]
        socket: PathBuf,
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

        /// The path to the exact input file
        #[clap(short, long)]
        exact: Option<PathBuf>,
    },

    /// Run trace test via the given trace file
    Tx {
        /// The path to the trace file
        test: PathBuf,
    },
}

impl Fuzz {
    /// Run the fuzz command
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Target { socket } => Target::serve(socket),
            Self::Fuzzer {
                socket,
                traces,
                report,
                exact,
            } => {
                if let Some(exact) = exact {
                    Fuzzer::execute(socket, exact, report)
                } else {
                    Fuzzer::run(socket, traces, report)
                }
            }
            Self::Tx { test } => fuzz::trace::test(test),
        }
    }
}
