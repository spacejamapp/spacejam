//! Authoring service

use network::{Event, Network};
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
    let (block, ticket) = runtime.next()?;
    if let Some(block) = block {
        tracing::info!(
            "subscribing block@{}: {}",
            block.header.slot,
            hex::encode(block.hash()?)
        );

        // save the block to the storage
        runtime.storage.save_block(&block)?;
        runtime
            .grandpa
            .write()
            .await
            .save_header(block.header.clone())?;

        // announce the block to the network
        runtime.send(Event::AnnounceBlock {
            header: Box::new(block.header.clone()),
            head: runtime.grandpa.read().await.head.clone(),
        })?;
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
