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

/// Receive an audit announcement.
pub async fn recv<C: runtime::Config>(
    send: SendStream,
    recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    Ok(())
}
