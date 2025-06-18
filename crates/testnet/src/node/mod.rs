//! Node implementations.

use crate::{
    log::{Message, Stream},
    Arch, Network, Node,
};
use anyhow::Result;
use std::{
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread,
};

mod polkajam;
mod spacejam;

impl Node {
    /// Build the node command and send logs through the provided channel.
    pub fn spawn(
        &self,
        network: &Network,
        name: &str,
        tx: mpsc::Sender<Message>,
    ) -> Result<NamedChild> {
        let mut cmd = self.command(network)?;
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = cmd.spawn()?;

        // Get stdout and stderr handles
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stderr"))?;

        let name = name.to_string();
        let filters = self.filter.clone();
        let stdout_sender = tx.clone();
        let stdout_name = name.clone();
        let stdout_filters = filters.clone();
        thread::spawn(move || {
            Self::send_message(
                BufReader::new(stdout),
                stdout_name,
                Stream::Stdout,
                stdout_filters,
                stdout_sender,
            );
        });

        let stderr_sender = tx.clone();
        let stderr_name = name.clone();
        let stderr_filters = filters.clone();
        thread::spawn(move || {
            Self::send_message(
                BufReader::new(stderr),
                stderr_name,
                Stream::Stderr,
                stderr_filters,
                stderr_sender,
            );
        });

        Ok(NamedChild { name, child })
    }

    /// Build the node command
    pub fn command(&self, network: &Network) -> Result<Command> {
        match self.arch {
            Arch::Polkajam => self.polkajam(network),
            Arch::SpaceJam => self.spacejam(network),
        }
    }

    /// Send message to channel if it matches the filters
    fn send_message<R>(
        reader: BufReader<R>,
        name: String,
        stream: Stream,
        filters: Vec<String>,
        sender: mpsc::Sender<Message>,
    ) where
        R: std::io::Read,
    {
        for line in reader.lines() {
            if let Ok(line) = line {
                if !filters.is_empty() && !filters.iter().any(|filter| line.contains(filter)) {
                    continue;
                }

                let log_msg = Message {
                    name: name.clone(),
                    stream,
                    content: line,
                };

                let _ = sender.send(log_msg);
            }
        }
    }
}

/// A child process that would be killed on drop.
pub struct NamedChild {
    /// The name of the child process.
    pub name: String,

    /// The child process.
    child: Child,
}

impl NamedChild {
    /// Check if the child process has terminated.
    pub fn terminated(&mut self) -> bool {
        self.child.wait().is_err()
    }
}

impl Drop for NamedChild {
    fn drop(&mut self) {
        self.child
            .kill()
            .expect(&format!("failed to kill {} process", self.name));
    }
}
