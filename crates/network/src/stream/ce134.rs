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

/// Receive a work package sharing.
pub async fn recv<C: runtime::Config>(
    send: SendStream,
    recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    Ok(())
}
