//! Authoring service

use crate::node::Context;
use network::event::action::Event;
use score::runtime::{storage::BlockStorage, Storage, Validator};
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

        // 1. save the block to the storage
        context.runtime.storage.save_block(&block)?;

        // 2. announce the block to the network
        let mut announcement = codec::encode(&block.header)?;
        announcement.extend_from_slice(&block.header.hash()?);
        announcement.extend_from_slice(&block.header.slot.to_le_bytes());
        context.tx.send(Event::AnnounceBlock(announcement).into())?;

        // TODO: currently we don't have a way to get the finalized header
        // from the runtime. so we update the newly produced block as the
        // finalized header of the chain.
        context.runtime.grandpa.write().await.update(block.header);
    }

    if let Some(ticket) = ticket {
        tracing::info!(
            "subscribing ticket@{}: {}",
            ticket.attempt,
            hex::encode(ticket.signature)
        );

        // context
        //     .tx
        //     .send(Event::SubscribeTicket(codec::encode(&ticket)?))
        //     .await?;
    }

    Ok(())
}
