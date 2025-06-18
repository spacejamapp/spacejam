//! The polkajam binary interface

use crate::Node;
use anyhow::Result;
use std::process::Command;

impl Node {
    /// Build the polkajam command.
    pub fn polkajam(&self) -> Result<Command> {
        let mut command = Command::new(&self.command);
        command.envs(&self.env).args(&[
            "-p",
            "tiny",
            "-c",
            "dev",
            "run",
            "-d",
            &self.data.to_string_lossy(),
            "--dev-validator",
            &self.seed,
            "--port",
            &self.quic_port()?.to_string(),
            "--rpc-port",
            &self.rpc_port()?.to_string(),
        ]);

        Ok(command)
    }
}
