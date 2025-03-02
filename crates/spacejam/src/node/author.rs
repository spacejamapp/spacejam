//! Authoring service

use network::{event::action::Event, Network};
use score::runtime::storage::BlockStorage;
use std::time::Duration;

/// Author blocks (mocked)
pub async fn run<C: score::runtime::Config>(runtime: &Network<C>) {
    loop {
        tokio::time::sleep(Duration::from_secs(score::SLOT_PERIOD as u64)).await;
        if let Err(e) = inner(runtime).await {
            tracing::error!("failed to author block: {e}");
        }
    }
}

async fn inner<C: score::runtime::Config>(runtime: &Network<C>) -> anyhow::Result<()> {
    let (block, ticket) = runtime.runtime.next()?;
    if let Some(block) = block {
        tracing::info!(
            "subscribing block@{}: {}",
            block.header.slot,
            hex::encode(block.hash()?)
        );

        // 1. save the block to the storage
        runtime.runtime.storage.save_block(&block)?;

        // 2. announce the block to the network
        let mut announcement = codec::encode(&block.header)?;
        announcement.extend_from_slice(&block.header.hash()?);
        announcement.extend_from_slice(&block.header.slot.to_le_bytes());
        runtime
            .transport
            .tx
            .send(Event::AnnounceBlock(announcement).into())?;

        // TODO: currently we don't have a way to get the finalized header
        // from the runtime. so we update the newly produced block as the
        // finalized header of the chain.
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
