//! Tests for connections.

use crypto::ed25519;
use metrics::{Metrics, Peer};
use network::{peer::PeerId, transport, Address, Config, Context, Event, Network, RuntimeApi};
use spacejam_network::{self as network};
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::sync::mpsc;

/// Test Node
pub struct Node {
    metrics: Metrics,
    keypair: ed25519::KeyPair,
    tx: mpsc::UnboundedSender<Event>,
}

impl Context for Node {
    fn keypair(&self) -> Option<ed25519::KeyPair> {
        Some(self.keypair.clone())
    }

    fn metrics(&self) -> &Metrics {
        &self.metrics
    }

    fn grandpa(&self) -> score::runtime::Grandpa {
        Arc::new(Default::default())
    }

    fn tx(&self) -> mpsc::UnboundedSender<Event> {
        self.tx.clone()
    }
}

impl RuntimeApi for Node {}

impl Node {
    /// Create a new node
    pub async fn new(
        config: Config,
        metrics: Metrics,
        keypair: ed25519::KeyPair,
    ) -> (Network<Self>, mpsc::UnboundedReceiver<Event>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let node = Arc::new(Self {
            metrics,
            keypair,
            tx,
        });

        (
            Network::new(config, node.clone())
                .await
                .expect("failed to init network"),
            rx,
        )
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
            PeerId::from(key.verifying.as_bytes()),
        )
    });

    // create nodes

    let (alice, alice_rx) = Node::new(
        Config {
            address: aaddress.addr.clone(),
            ..Default::default()
        },
        Metrics::new("Alice"),
        akey,
    )
    .await;
    let (bob, bob_rx) = Node::new(
        Config {
            address: baddress.addr.clone(),
            bootstrap: vec![aaddress],
            ..Default::default()
        },
        Metrics::new("Bob"),
        bkey,
    )
    .await;

    let actx = alice.context.clone();
    tokio::select! {
        r = alice.spawn(alice_rx) => r,
        r = bob.spawn(bob_rx) => r,
        _ = async {
            let peer_ref = Peer {
                peer: baddress.to_string(),
            };

            loop {
                if let Some(conn) = actx.metrics().conn.get(&peer_ref) {
                    if conn.get() == Peer::established() {
                        break;
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        } => {},
    }
}
