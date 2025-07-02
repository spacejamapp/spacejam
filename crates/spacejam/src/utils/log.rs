//! logging utilities

use network::Network;
use runtime::Validator;
use score::block;

/// Logging the initial status of the node
pub async fn init<C: runtime::Config>(runtime: &Network<C>) {
    let chain = runtime.chain().await;
    let handshake = chain.grandpa.handshake.clone();
    tracing::info!(
        "The latest finalized head #{}: 0x{}",
        handshake.head.slot,
        hex::encode(handshake.head.hash)
    );
}

/// Logging the current status of the node
pub async fn current<C: runtime::Config>(runtime: &Network<C>) {
    let grid = runtime.grid().await;
    // TODO: handle this gracefully
    let validators = grid.curr;
    let pool = runtime.pool.read().await.clone();
    let peers = pool.keys().collect::<Vec<_>>();
    let connected = peers
        .iter()
        .filter(|p| validators.iter().any(|v| &v.ed25519 == p.as_ref()))
        .count() as u16;

    // check neighbours
    let (grid, handshake) = {
        let grid = runtime.grid().await;
        let handshake = runtime.handshake().await;
        (grid, handshake)
    };
    let neighbours = grid.neighbours(runtime.validator.ed25519_public_key());

    let connected_neighbours = pool
        .iter()
        .filter(|(peer, _conn)| neighbours.contains(peer.as_ref()))
        .count();
    let total_neighbours = neighbours.len();

    // get the latest pending block
    let tickets = runtime.tickets().await;

    // print the current status
    let timeslot = block::timeslot();
    let best = runtime.chain().await.best().unwrap_or_default();
    tracing::info!(
        "timeslot: #{}, epoch: #{}, progress: [{}/{}], best: #{}@0x{}, finalized: #{}@0x{}, tickets: {}",
        timeslot,
        timeslot / score::EPOCH_LENGTH,
        timeslot % score::EPOCH_LENGTH,
        score::EPOCH_LENGTH,
        best.slot,
        hex::encode(&best.hash[..3]),
        handshake.head.slot,
        hex::encode(&handshake.head.hash[..3]),
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
