//! The polkajam binary interface

use crate::{Network, Node};
use anyhow::Result;
use std::process::Command;

impl Node {
    /// Build the polkajam command.
    pub fn polkajam(&self, net: &Network) -> Result<Command> {
        let mut command = Command::new(&self.command);
        command.envs(&self.env).args([
            "-c",
            "dev",
            "--chain",
            &net.spec.to_string_lossy(),
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
