//! Tests for connections.

use crypto::ed25519;
use metrics::{Metrics, Peer};
use network::{peer::PeerId, Config};
use spacejam_network::{self as network, Address, Context, Handle, Manager};
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::sync::{mpsc, RwLock};

/// Test Node
pub struct Node {
    manager: Arc<RwLock<Manager>>,
    metrics: Metrics,
    keypair: ed25519::KeyPair,
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

    fn manager(&self) -> Arc<tokio::sync::RwLock<spacejam_network::Manager>> {
        self.manager.clone()
    }
}

impl Node {
    /// Create a new node
    pub async fn new(
        config: Config,
        metrics: Metrics,
        keypair: ed25519::KeyPair,
    ) -> (Arc<Self>, Handle<Self>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let manager = Arc::new(RwLock::new(Manager::new(tx)));
        let node = Arc::new(Self {
            metrics,
            manager,
            keypair,
        });
        let handle = Handle {
            rx,
            context: node.clone(),
        };

        network::init(config, node.clone())
            .await
            .expect("failed to init network");
        (node, handle)
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
                network::pick().expect("failed to pick port"),
            ),
            PeerId::from(key.verifying.as_bytes()),
        )
    });

    // create nodes
    let (alice, ahandle) = Node::new(
        Config {
            address: aaddress.addr.clone(),
            ..Default::default()
        },
        Metrics::new("Alice"),
        akey,
    )
    .await;
    let (_, bhandle) = Node::new(
        Config {
            address: baddress.addr.clone(),
            bootstrap: vec![aaddress],
            ..Default::default()
        },
        Metrics::new("Bob"),
        bkey,
    )
    .await;

    tokio::select! {
        r = ahandle.spawn() => r,
        r = bhandle.spawn() => r,
        _ = async {
            let peer_ref = Peer {
                peer: baddress.to_string(),
            };

            loop {
                if let Some(conn) = alice.metrics().conn.get(&peer_ref) {
                    if conn.get() == Peer::established() {
                        break;
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        } => {},
    }
}
