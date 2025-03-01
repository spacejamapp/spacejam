//! Shard distribution stream.

use crate::Network;
use quinn::{RecvStream, SendStream};

/// Send a shard distribution.
pub async fn send<C: score::runtime::Config>(
    send: SendStream,
    recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    Ok(())
}

/// Receive a shard distribution.
pub async fn recv<C: score::runtime::Config>(
    send: SendStream,
    recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    Ok(())
}
