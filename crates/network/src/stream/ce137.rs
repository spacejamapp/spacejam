//! Shard distribution stream.

use crate::Network;
use quinn::{RecvStream, SendStream};

impl<C: runtime::Config> Network<C> {
    /// Receive a shard distribution.
    pub async fn recv_ce137(&self, _send: SendStream, _recv: RecvStream) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Send a shard distribution.
#[allow(unused)]
pub async fn send<C: runtime::Config>(
    send: SendStream,
    recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    Ok(())
}
