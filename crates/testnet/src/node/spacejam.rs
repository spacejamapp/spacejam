//! The spacejam binary interface

use crate::Node;
use anyhow::Result;
use std::process::Command;

impl Node {
    /// Build the spacejam command.
    pub fn spacejam(&self) -> Result<Command> {
        let mut command = Command::new(&self.command);
        command.envs(&self.env).args(&[
            "-d",
            &self.data.to_string_lossy(),
            "run",
            "--validator",
            &self.seed,
            "--address",
            &self.quic,
        ]);
        command.args(&self.args);
        Ok(command)
    }
}
