//! Segment shard request stream (with justification).

use crate::Network;
use quinn::{RecvStream, SendStream};

/// Send a segment shard request.
pub async fn send<C: runtime::Config>(
    send: SendStream,
    recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    Ok(())
}

impl<C: runtime::Config> Network<C> {
    /// Receive a segment shard request.
    pub async fn recv_ce140(&self, _send: SendStream, _recv: RecvStream) -> anyhow::Result<()> {
        Ok(())
    }
}
