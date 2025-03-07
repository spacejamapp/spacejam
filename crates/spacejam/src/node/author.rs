//! Authoring service

use network::{Event, Network};
use score::{block, runtime::storage::BlockStorage};
use std::time::Duration;

/// Author blocks (mocked)
pub async fn run<C: score::runtime::Config>(runtime: &Network<C>) {
    loop {
        if !runtime.is_validator().await {
            let Ok(timeslot) = block::timeslot() else {
                tracing::error!("failed to get timeslot");
                break;
            };

            let dur = timeslot - (timeslot % score::EPOCH_LENGTH);
            tokio::time::sleep(Duration::from_secs(dur as u64)).await;
            continue;
        }

        // author a block
        if let Err(e) = inner(runtime).await {
            tracing::warn!("failed to author block: {e}");
        }

        // wait for the next slot, mb we can trigger this in the network event loop?
        tokio::time::sleep(Duration::from_secs(score::SLOT_PERIOD as u64)).await;
    }
}

async fn inner<C: score::runtime::Config>(runtime: &Network<C>) -> anyhow::Result<()> {
    let (block, ticket) = runtime.next().await?;
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

    if let Some(ticket) = ticket {
        tracing::info!(
            "subscribing ticket@{}: {}",
            ticket.attempt,
            hex::encode(ticket.signature)
        );

        let epoch = block.header.slot / score::EPOCH_LENGTH;
        runtime.send(Event::DistributeTicket {
            epoch,
            ticket: Box::new(ticket),
        })?;
    }

    Ok(())
}
