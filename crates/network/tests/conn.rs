//! Tests for connections.

use crypto::ed25519;
use metrics::Metrics;
use network::{peer::PeerId, Config, Network};
use spacejam_network as network;
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::sync::mpsc;

#[tokio::test]
async fn connections() -> anyhow::Result<()> {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Create channels with proper senders to keep them alive
    let (_atx, arx) = mpsc::unbounded_channel();
    let (_btx, brx) = mpsc::unbounded_channel();
    let aport = network::pick()?;
    let localhost = Ipv4Addr::new(127, 0, 0, 1);
    let address = SocketAddr::new(localhost.into(), aport);

    let akey = ed25519::KeyPair::from([0; 32]);
    let maddress = (address, PeerId::from(akey.verifying.as_bytes())).into();
    let alice = Network::new(
        Config {
            address,
            ..Default::default()
        },
        arx,
        Some(akey.clone()),
    )
    .await?;

    let bkey = ed25519::KeyPair::from([1; 32]);
    let bob = Network::new(
        Config {
            address: SocketAddr::new(localhost.into(), network::pick()?),
            bootstrap: vec![maddress],
            genesis: [0; 32],
        },
        brx,
        Some(bkey.clone()),
    )
    .await?;

    let apeer = PeerId::from(akey.verifying.as_bytes());
    let bpeer = PeerId::from(bkey.verifying.as_bytes());
    tokio::select! {
        r = alice.spawn(Arc::new(Metrics::new(&apeer.to_string()))) => r,
        r = bob.spawn(Arc::new(Metrics::new(&bpeer.to_string()))) => r,
    };

    Ok(())
}
