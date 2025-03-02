//! Audit shard request stream.

use crate::Network;
use quinn::{RecvStream, SendStream};

/// Send an audit shard request.
pub async fn send<C: score::runtime::Config>(
    send: SendStream,
    recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    Ok(())
}

/// Receive an audit shard request.
pub async fn recv<C: score::runtime::Config>(
    send: SendStream,
    recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    Ok(())
}
