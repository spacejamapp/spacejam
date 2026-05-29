//! Fuzz related commands

use crate::fuzz::target::Target;
#[cfg(feature = "trace")]
use crate::fuzz::{self, fuzzer::Fuzzer};
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

        /// If use interpreter instead
        #[clap(short, long, env = "SPACEJAM_INTERP")]
        interp: bool,
    },

    /// Fuzz with a fuzzer
    #[cfg(feature = "trace")]
    Fuzzer {
        /// The path to the fuzzer
        #[clap(default_value = "/tmp/jam_target.sock")]
        socket: PathBuf,

        /// The path to the traces folder
        #[clap(default_value = "jam-test-vectors/traces/storage", short, long)]
        traces: PathBuf,

        /// The path to the conformance folder
        #[clap(short, long)]
        conformance: Option<PathBuf>,

        /// The path to the report folder
        #[clap(default_value = "reports", short, long)]
        report: PathBuf,

        /// The path to the exact input file
        #[clap(short, long)]
        exact: Option<PathBuf>,
    },

    /// Run trace test via the given trace file or directory
    #[cfg(feature = "trace")]
    Tx {
        /// The path to the trace file or directory of `.bin`/`.json` traces
        test: PathBuf,
    },
}

impl Fuzz {
    /// Run the fuzz command
    pub async fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Target { socket, interp } => Target::serve(socket, *interp).await,
            #[cfg(feature = "trace")]
            Self::Fuzzer {
                socket,
                traces,
                conformance,
                report,
                exact,
            } => {
                if let Some(exact) = exact {
                    Fuzzer::execute(socket, exact, report)
                } else if let Some(conformance) = conformance {
                    Fuzzer::conformance(socket, conformance, report)
                } else {
                    Fuzzer::run(socket, traces, report)
                }
            }
            #[cfg(feature = "trace")]
            Self::Tx { test } => {
                if test.is_dir() {
                    fuzz::trace::test_dir(test).await
                } else {
                    fuzz::trace::test(test).await
                }
            }
        }
    }
}
