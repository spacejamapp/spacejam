//! logging utilities

use network::Network;
use score::{
    block,
    runtime::{storage::BlockStorage, Storage, Validator},
    safrole::ValidatorData,
};

/// Logging the initial status of the node
pub async fn init<C: score::runtime::Config>(runtime: &Network<C>) {
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

/// Logging the current status of the node
pub async fn current<C: score::runtime::Config>(
    runtime: &Network<C>,
    validators: &[ValidatorData],
) {
    let pool = runtime.pool.read().await.clone();
    let peers = pool.keys().collect::<Vec<_>>();
    let connected = peers
        .iter()
        .filter(|p| validators.iter().any(|v| &v.ed25519 == p.as_ref()))
        .count() as u16
        + 1;

    // check neighbours
    let grandpa = runtime.grandpa.read().await.clone();
    let neighbours = grandpa
        .grid
        .neighbours(runtime.validator.ed25519_public_key());
    let connected_neighbours = pool
        .iter()
        .filter(|(peer, conn)| neighbours.contains(peer.as_ref()) && conn.ready())
        .count();
    let total_neighbours = neighbours.len();

    // get the latest pending block
    let (pending, tickets) = {
        let chain = runtime.chain().await;
        (
            chain.get_finalized().unwrap_or_default(),
            chain.safrole().unwrap_or_default().accumulator.len(),
        )
    };

    // print the current status
    let timeslot = block::timeslot().unwrap_or_default();
    tracing::info!(
        "epoch: #{}, progress: [{}/{}], pending: #{}@0x{}, grandpa: #{}@0x{}, tickets: {}",
        timeslot / score::EPOCH_LENGTH,
        timeslot % score::EPOCH_LENGTH,
        score::EPOCH_LENGTH,
        pending.slot,
        hex::encode(&pending.hash[..3]),
        grandpa.handshake.head.slot,
        hex::encode(&grandpa.handshake.head.hash[..3]),
        tickets,
    );
    tracing::debug!(
        "peers: {}, connected validators: [{}/{}], connected neighbours: [{}/{}]",
        peers.len(),
        connected,
        score::VALIDATORS_COUNT,
        connected_neighbours,
        total_neighbours
    );
}
