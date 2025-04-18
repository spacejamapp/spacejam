//! Work package sharing stream.

use crate::Network;
use quinn::{RecvStream, SendStream};

/// Send a work package sharing.
pub async fn send<C: runtime::Config>(
    send: SendStream,
    recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    Ok(())
}

impl<C: runtime::Config> Network<C> {
    /// Receive a work package sharing.
    pub async fn recv_ce134(&self, _send: SendStream, _recv: RecvStream) -> anyhow::Result<()> {
        Ok(())
    }
}
