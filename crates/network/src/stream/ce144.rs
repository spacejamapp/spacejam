//! Audit announcement stream.

use crate::Network;
use quinn::{RecvStream, SendStream};

/// Send an audit announcement.
pub async fn send<C: runtime::Config>(
    send: SendStream,
    recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    Ok(())
}

impl<C: runtime::Config> Network<C> {
    /// Receive an audit announcement.
    pub async fn recv_ce144(&self, _send: SendStream, _recv: RecvStream) -> anyhow::Result<()> {
        Ok(())
    }
}
