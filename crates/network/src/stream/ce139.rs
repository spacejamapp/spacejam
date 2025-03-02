//! Segment shard request stream.

use crate::Network;
use quinn::{RecvStream, SendStream};

/// Send a segment shard request.
pub async fn send<C: score::runtime::Config>(
    send: SendStream,
    recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    Ok(())
}

/// Receive a segment shard request.
pub async fn recv<C: score::runtime::Config>(
    send: SendStream,
    recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    Ok(())
}
