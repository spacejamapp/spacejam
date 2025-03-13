//! Authoring service

use network::{Event, Network};
use score::{
    block,
    runtime::{storage::BlockStorage, tx, Head, Storage, Validator},
};
use std::time::Duration;

/// Author blocks
///
/// we should only start authoring if we have 2/3 validators connected.
#[tracing::instrument(skip_all, name = "authoring")]
pub async fn run<C: score::runtime::Config>(runtime: &Network<C>) {
    {
        let grandpa = runtime.grandpa.read().await;
        tracing::info!(
            "The latest finalized head #{}: 0x{}",
            grandpa.handshake.head.slot,
            hex::encode(grandpa.handshake.head.hash)
        );

        let chain = runtime.chain().await;
        if let Ok(block) = chain.get_finalized() {
            tracing::info!(
                "The latest pending block #{}: 0x{}",
                block.slot,
                hex::encode(block.hash)
            );
        }
    }

    loop {
        let validators = runtime.grandpa.read().await.grid.curr;
        {
            // if we are not in the safrole series keys, we should not author blocks
            let Ok(safrole) = runtime.chain().await.safrole() else {
                tracing::error!("failed to get safrole state");
                break;
            };

            let series = safrole.series.keys();
            if !series.is_empty() && !series.contains(&runtime.validator.bandersnatch_public_key())
            {
                let Ok(timeslot) = block::timeslot() else {
                    tracing::error!("failed to get timeslot");
                    break;
                };

                let dur = timeslot - (timeslot % score::EPOCH_LENGTH);
                tracing::info!(
                    "not in the safrole series keys {:#?}, sleeping for authoring till next epoch",
                    series
                        .into_iter()
                        .map(|key| format!("0x{}", hex::encode(key)))
                        .collect::<Vec<_>>()
                );
                tokio::time::sleep(Duration::from_secs(dur as u64)).await;
                continue;
            }
        }

        // if we haven't seen 2/3 of the validators, we should not author blocks
        {
            let pool = runtime.pool.read().await.clone();
            let peers = pool.keys().collect::<Vec<_>>();
            let connected = peers
                .iter()
                .filter(|p| validators.contains(p.as_ref()))
                .count() as u16
                + 1;

            let grandpa = runtime.grandpa.read().await.clone();
            let neighbours = grandpa
                .grid
                .neighbours(runtime.validator.ed25519_public_key());
            let connected_neighbours = pool
                .iter()
                .filter(|(peer, conn)| neighbours.contains(peer.as_ref()) && conn.ready())
                .count();
            let total_neighbours = neighbours.len();

            tracing::debug!(
                "grandpa: #{}, peers: {}, connected validators: [{}/{}], connected neighbours: [{}/{}]",
                grandpa.handshake.head.slot,
                peers.len(),
                connected,
                score::VALIDATORS_COUNT,
                connected_neighbours,
                total_neighbours
            );
            if connected < score::VALIDATORS_SUPER_MAJORITY
                || connected_neighbours != total_neighbours
            {
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
    let (mut block, ticket) = runtime.next().await?;
    let chain = runtime.chain().await;

    // save the block to the storage
    let head = Head {
        hash: block.hash()?,
        slot: block.header.slot,
    };
    chain.save_block(&block)?;
    chain.set_finalized(&head)?;
    // let safrole = chain.safrole()?;
    tracing::debug!(
        "tickets count: {}",
        runtime.expool.tickets.lock().await.len()
    );

    // save the header to the grandpa
    {
        let mut grandpa = runtime.grandpa.write().await;
        grandpa.add_leaf(block.header.clone())?;
    }

    // transit the state
    tx::transit(&mut block, &chain, &runtime.validator)?;

    // announce the block to the network
    runtime.send(Event::AnnounceBlock(Box::new(block.header.clone())))?;

    if let Some(ticket) = ticket {
        let epoch = block.header.slot / score::EPOCH_LENGTH;
        runtime.send(Event::DistributeTicket {
            epoch,
            ticket: Box::new(ticket),
        })?;
    }

    Ok(())
}
