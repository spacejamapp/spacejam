use metrics::Peer;
use spacejam_network::{Config, Network};
use std::time::Duration;

#[tokio::test]
async fn handshake_locally() {
    let config = Config::default();
    let mut alice = Network::new(config.clone())
        .await
        .expect("failed to create alice network");

    let mut bob = Network::new(config)
        .await
        .expect("failed to create bob network");

    let ametrics = alice.metrics.clone();
    let bmetrics = bob.metrics.clone();
    let alice_address = alice.p2p.local_peer_id().to_string();
    let bob_address = bob.p2p.local_peer_id().to_string();

    tokio::select! {
        _ = alice.spawn() => {}
        _ = bob.spawn() => {}
        _ = async {
            if let (Some(aconn), Some(bconn)) = (ametrics.connections.get(&Peer { peer: alice_address }), bmetrics.connections.get(&Peer { peer: bob_address })) {
                if aconn.get() == Peer::established() && bconn.get() == Peer::established() {
                    return;
                }
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        } => {}
    }
}
