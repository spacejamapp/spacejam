//! Tests for connections.

use crypto::ed25519;
use metrics::{Metrics, Peer};
use network::{peer::PeerId, Config, Network};
use spacejam_network::{self as network, Address};
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::sync::mpsc;

#[tokio::test]
async fn connections() {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Create channels with proper senders to keep them alive
    let (_atx, arx) = mpsc::unbounded_channel();
    let (_btx, brx) = mpsc::unbounded_channel();
    let localhost = Ipv4Addr::new(127, 0, 0, 1);

    let [akey, bkey] = [
        ed25519::KeyPair::from([0; 32]),
        ed25519::KeyPair::from([1; 32]),
    ];
    let [aaddress, baddress] = [akey.clone(), bkey.clone()].map(|key| {
        Address::new(
            SocketAddr::new(
                localhost.into(),
                network::pick().expect("failed to pick port"),
            ),
            PeerId::from(key.verifying.as_bytes()),
        )
    });

    let (_ptx, prx) = mpsc::unbounded_channel();
    let actx = Arc::new(Metrics::new("Alice"));
    let alice = Network::new(
        Config {
            address: aaddress.addr.clone(),
            ..Default::default()
        },
        actx.clone(),
        arx,
        prx,
    )
    .await
    .expect("failed to create alice");

    let (_ptx, prx) = mpsc::unbounded_channel();
    let bctx = Arc::new(Metrics::new("Bob"));
    let bob = Network::new(
        Config {
            address: baddress.addr,
            bootstrap: vec![aaddress],
            genesis: [0; 32],
        },
        bctx.clone(),
        brx,
        prx,
    )
    .await
    .expect("failed to create bob");

    tokio::select! {
        r = alice.spawn(actx.clone()) => r,
        r = bob.spawn(bctx.clone()) => r,
        _ = async {
            let peer_ref = Peer {
                peer: baddress.to_string(),
            };

            loop {
                if let Some(conn) = actx.conn.get(&peer_ref) {
                    if conn.get() == Peer::established() {
                        break;
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        } => {},
    }
}
