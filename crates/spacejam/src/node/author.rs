//! Authoring service

use crate::node::{Context, Event};
use score::runtime::{Storage, Validator};
use std::time::Duration;

/// Author blocks (mocked)
pub async fn run<S: Storage, V: Validator>(context: &Context<S, V>) {
    loop {
        tokio::time::sleep(Duration::from_secs(score::SLOT_PERIOD as u64)).await;
        if let Err(e) = inner(context).await {
            tracing::error!("failed to author block: {e}");
        }
    }
}

async fn inner<S: Storage, V: Validator>(context: &Context<S, V>) -> anyhow::Result<()> {
    let (block, ticket) = context.runtime.next()?;
    if let Some(block) = block {
        tracing::info!(
            "subscribing block@{}: {}",
            block.header.slot,
            hex::encode(block.hash()?)
        );

        context
            .tx
            .send(Event::SubscribeBlock(codec::encode(&block)?))
            .await?;
    }

    if let Some(ticket) = ticket {
        tracing::info!(
            "subscribing ticket@{}: {}",
            ticket.attempt,
            hex::encode(ticket.signature)
        );

        context
            .tx
            .send(Event::SubscribeTicket(codec::encode(&ticket)?))
            .await?;
    }

    Ok(())
}
