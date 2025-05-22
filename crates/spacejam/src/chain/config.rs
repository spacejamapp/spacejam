//! Chain Configurations

use serde::{Deserialize, Serialize};

/// Chain Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Chain ID
    id: String,
    /// Genesis Validators
    genesis_validators: [GenesisValidator; score::VALIDATORS_COUNT as usize],
}

/// Genesis Validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisValidator {
    /// Peer ID
    peer_id: String,

    /// Bandersnatch Public Key
    bandersnatch: String,

    /// Network Address
    net_addr: String,
}
