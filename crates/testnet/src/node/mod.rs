//! Node implementations.

use crate::{
    config::Filter,
    log::{Message, Stream},
    Arch, Network, Node,
};
use anyhow::Result;
use std::{
    io::{BufRead, BufReader, Read},
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

        self.spawn_logging(name.to_string(), tx.clone(), stdout, Stream::Stdout);
        self.spawn_logging(name.to_string(), tx, stderr, Stream::Stderr);
        Ok(NamedChild {
            name: name.to_string(),
            child,
        })
    }

    /// Build the node command
    pub fn command(&self, network: &Network) -> Result<Command> {
        match self.arch {
            Arch::Polkajam => self.polkajam(network),
            Arch::SpaceJam => self.spacejam(network),
        }
    }

    fn spawn_logging<R: Read + Send + 'static>(
        &self,
        name: String,
        tx: mpsc::Sender<Message>,
        reader: R,
        stream: Stream,
    ) {
        let arch = self.arch;
        let filter = self.filter.clone();
        thread::spawn(move || {
            Self::send_message(BufReader::new(reader), arch, name, stream, filter, tx);
        });
    }

    /// Send message to channel if it matches the filters
    fn send_message<R>(
        reader: BufReader<R>,
        arch: Arch,
        name: String,
        stream: Stream,
        filter: Filter,
        tx: mpsc::Sender<Message>,
    ) where
        R: std::io::Read,
    {
        for line in reader.lines().map_while(Result::ok) {
            if filter.skip(&line) {
                continue;
            }

            let log_msg = Message {
                arch,
                name: name.clone(),
                stream,
                content: line,
            };

            let _ = tx.send(log_msg);
        }

        tx.send(Message {
            arch,
            name,
            stream: Stream::Terminated,
            content: "".to_string(),
        })
        .expect("failed to send terminated message");
    }
}

/// A child process that would be killed on drop.
pub struct NamedChild {
    /// The name of the child process.
    name: String,

    /// The child process.
    child: Child,
}

impl Drop for NamedChild {
    fn drop(&mut self) {
        self.child
            .kill()
            .unwrap_or_else(|_| eprintln!("failed to kill {} process", self.name));
    }
}
