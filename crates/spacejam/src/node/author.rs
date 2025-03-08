//! Authoring service

use network::{Event, Network};
use score::{
    block,
    runtime::{storage::BlockStorage, Validator},
};
use std::time::Duration;

/// Author blocks
///
/// we should only start authoring if we have 2/3 validators connected.
pub async fn run<C: score::runtime::Config>(runtime: &Network<C>) {
    loop {
        let validators = runtime.grandpa.read().await.grid.curr;

        // if we are not a validator, we should not author blocks
        if !validators.contains(&runtime.validator.ed25519_public_key()) {
            tracing::debug!("not a validator, sleeping for authoring till next epoch");
            let Ok(timeslot) = block::timeslot() else {
                tracing::error!("failed to get timeslot");
                break;
            };

            let dur = timeslot - (timeslot % score::EPOCH_LENGTH);
            tokio::time::sleep(Duration::from_secs(dur as u64)).await;
            continue;
        }

        // if we haven't seen 2/3 of the validators, we should not author blocks
        {
            let pool = runtime.pool.read().await;
            let peers = pool.keys().collect::<Vec<_>>().clone();
            let connected = peers
                .iter()
                .filter(|p| validators.contains(p.as_ref()))
                .count() as u16;

            tracing::debug!(
                "connected validators: [{}/{}]",
                connected,
                score::VALIDATORS_COUNT
            );
            if connected < score::VALIDATORS_SUPER_MAJORITY {
                tokio::time::sleep(Duration::from_secs(score::SLOT_PERIOD as u64)).await;
                continue;
            }
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
        let epoch = block.header.slot / score::EPOCH_LENGTH;
        runtime.send(Event::DistributeTicket {
            epoch,
            ticket: Box::new(ticket),
        })?;
    }

    Ok(())
}
