//! Configuration for the network.

use serde::{Deserialize, Serialize};
use std::{net::Ipv4Addr, time::Duration};

/// Configuration for the network.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "cmd", derive(clap::Parser))]
pub struct Config {}
