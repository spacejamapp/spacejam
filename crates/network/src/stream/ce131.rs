//! Safrole ticket distribution stream (first step).

use crate::Context;
use quinn::{RecvStream, SendStream};
use std::sync::Arc;

/// Send a safrole ticket distribution.
pub async fn send<C: Context>(
    send: SendStream,
    recv: RecvStream,
    context: Arc<C>,
) -> anyhow::Result<()> {
    Ok(())
}

/// Receive a safrole ticket distribution.
pub async fn recv<C: Context>(
    send: SendStream,
    recv: RecvStream,
    context: Arc<C>,
) -> anyhow::Result<()> {
    Ok(())
}
