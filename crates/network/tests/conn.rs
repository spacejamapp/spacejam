//! Tests for connections.

use crypto::ed25519;
use metrics::Peer;
use network::{peer::PeerId, transport, Address, Config, Network};
use spacejam_network::{self as network};
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

/// Test Node
pub struct Node {
    keypair: ed25519::KeyPair,
}

impl Node {
    /// Create a new node
    pub async fn new(config: Config, keypair: ed25519::KeyPair) -> Network<()> {
        let node = Arc::new(Self { keypair });
        let runtime = Arc::new(runtime::Runtime::new(
            node.keypair.clone(),
            runtime::storage::MemoryDb::default(),
            (),
        ));

        Network::new(config, runtime)
            .await
            .expect("failed to init network")
    }
}

#[tokio::test]
async fn connections() {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Create channels with proper senders to keep them alive
    let localhost = Ipv4Addr::new(127, 0, 0, 1);
    let [akey, bkey] = [
        ed25519::KeyPair::from([0; 32]),
        ed25519::KeyPair::from([1; 32]),
    ];
    let [aaddress, baddress] = [akey.clone(), bkey.clone()].map(|key| {
        Address::new(
            SocketAddr::new(
                localhost.into(),
                transport::pick().expect("failed to pick port"),
            ),
            PeerId::from(key.verifying.to_bytes()),
        )
    });

    // create nodes
    let alice = Node::new(
        Config {
            address: aaddress.addr.clone(),
            ..Default::default()
        },
        akey,
    )
    .await;
    let bob = Node::new(
        Config {
            address: baddress.addr.clone(),
            bootstrap: vec![aaddress],
            ..Default::default()
        },
        bkey,
    )
    .await;

    let ametrics = alice.metrics.clone();
    let ahandle = alice.spawn();
    tokio::select! {
        r = ahandle => r,
        r = bob.spawn() => r,
        _ = async {
            let peer_ref = Peer {
                peer: baddress.to_string(),
            };

            loop {
                if let Some(conn) = ametrics.conn.get(&peer_ref) {
                    if conn.get() == Peer::established() {
                        break;
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        } => {},
    }
}
