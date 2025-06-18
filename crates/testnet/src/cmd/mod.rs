//! Command line interface for testnet.

use crate::{Message, Network, Testnet};
use clap::Parser;
use colored::Colorize;
use std::{fs, path::PathBuf, sync::mpsc};

/// The command line interface for testnet.
#[derive(Parser)]
pub struct App {
    /// The path to the testnet configuration file.
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Whether to use ANSI colors in the output.
    #[arg(short, long)]
    pub no_ansi: bool,
}

impl App {
    /// Run the testnet.
    pub fn run(self) -> anyhow::Result<()> {
        let testnet: Testnet = if let Some(config) = self.config {
            toml::from_str(&fs::read_to_string(&config)?)?
        } else {
            Testnet::default()
        };

        // spawn the nodes
        let (tx, rx) = mpsc::channel();
        let mut children = Vec::new();
        for (name, node) in testnet.node {
            let tx = tx.clone();
            let child = node.spawn(&testnet.network, &name, tx).map_err(|e| {
                eprintln!("failed to spawn node {}: {}", name, e);
                e
            })?;
            children.push(child);
        }

        Self::logging(rx, &testnet.network, self.no_ansi);
        Ok(())
    }

    /// Log messages from the nodes.
    fn logging(rx: mpsc::Receiver<Message>, network: &Network, no_ansi: bool) {
        while let Ok(msg) = rx.recv() {
            if !network.filter.is_empty() && !network.filter.iter().any(|f| msg.content.contains(f))
            {
                continue;
            }

            if !network.watch.is_empty() && !network.watch.contains(&msg.name) {
                continue;
            }

            if !no_ansi {
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
