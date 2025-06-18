//! Command line interface for testnet.

use crate::{log::Stream, Message, Network, Testnet};
use clap::Parser;
use colored::Colorize;
use std::{fs, path::PathBuf, sync::mpsc};

/// The command line interface for testnet.
#[derive(Parser)]
pub struct App {
    /// The command to run.
    #[command(subcommand)]
    command: Command,

    /// The path to the testnet configuration file.
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Whether to use ANSI colors in the output.
    #[arg(short, long)]
    pub noansi: bool,
}

impl App {
    /// Run the testnet.
    pub fn run(self) -> anyhow::Result<()> {
        let testnet: Testnet = if let Some(config) = &self.config {
            toml::from_str(&fs::read_to_string(config)?)?
        } else {
            Testnet::default()
        };

        match self.command {
            Command::Generate => {
                let testnet = Testnet::default();
                let toml = toml::to_string(&testnet)?;
                println!("{toml}");
                Ok(())
            }
            Command::Prune => testnet.prune(),
            Command::Start { prune } => self.start(testnet, prune),
        }
    }

    fn start(self, testnet: Testnet, prune: bool) -> anyhow::Result<()> {
        if testnet.node.is_empty() {
            anyhow::bail!("no nodes found in the testnet configuration");
        }

        if prune {
            testnet.prune()?;
        }

        // spawn the nodes
        let (tx, rx) = mpsc::channel();
        let mut children = Vec::new();
        for (name, node) in testnet.node {
            let tx = tx.clone();
            let child = node.spawn(&testnet.network, &name, tx).inspect_err(|_e| {
                eprintln!("failed to spawn node {name}");
            })?;
            children.push(child);
        }

        Self::logging(rx, &testnet.network, self.noansi);
        Ok(())
    }

    /// Log messages from the nodes.
    fn logging(rx: mpsc::Receiver<Message>, network: &Network, ansi: bool) {
        while let Ok(msg) = rx.recv() {
            if msg.stream == Stream::Terminated {
                eprintln!("{} terminated", msg.name);
                std::process::exit(1);
            }

            if !network.filter.is_empty() && !network.filter.iter().any(|f| msg.content.contains(f))
            {
                continue;
            }

            if !network.watch.is_empty() && !network.watch.contains(&msg.name) {
                continue;
            }

            if ansi {
                println!(
                    "{} {}",
                    if msg.content.contains("ERROR") {
                        msg.name.on_bright_red().bold().white()
                    } else if msg.content.contains("WARN") {
                        msg.name.on_bright_yellow().bold().black()
                    } else {
                        msg.name.bright_white().bold()
                    },
                    msg.content
                );
            } else {
                println!("{} {}", msg.name, msg.content);
            }
        }
    }
}

/// The command to run.
#[derive(Parser)]
pub enum Command {
    /// Generate a new testnet configuration file.
    Generate,
    /// Prune the testnet.
    Prune,
    /// Start the testnet.
    Start {
        /// Whether to prune the testnet.
        #[arg(short, long)]
        prune: bool,
    },
}
