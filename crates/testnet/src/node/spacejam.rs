//! The spacejam binary interface

use crate::{Network, Node};
use anyhow::Result;
use std::process::Command;

impl Node {
    /// Build the spacejam command.
    pub fn spacejam(&self, net: &Network) -> Result<Command> {
        let mut command = Command::new(&self.command);
        command.envs(&self.env).args(&self.args).args([
            "run",
            "-d",
            &self.data.to_string_lossy(),
            "--chain",
            &net.spec.to_string_lossy(),
            "--validator",
            &self.seed,
            "--address",
            &self.quic,
        ]);
        Ok(command)
    }
}
