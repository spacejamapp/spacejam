//! Tests for connections.

use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use crypto::ed25519;
use metrics::Metrics;
use network::{Config, Network};
use spacejam_network as network;
use tokio::sync::mpsc;

#[ignore]
#[tokio::test]
async fn connections() -> anyhow::Result<()> {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let (_, arx) = mpsc::unbounded_channel();
    let (_, brx) = mpsc::unbounded_channel();
    let aport = network::pick()?;
    let localhost = Ipv4Addr::new(127, 0, 0, 1);
    let address = SocketAddr::new(localhost.into(), aport);

    let alice = Network::new(
        Config {
            address,
            ..Default::default()
        },
        arx,
        Some(ed25519::KeyPair::from([0; 32])),
    )
    .await?;

    let bob = Network::new(
        Config {
            address: SocketAddr::new(localhost.into(), network::pick()?),
            bootstrap: vec![address],
            genesis: [0; 32],
        },
        brx,
        Some(ed25519::KeyPair::from([1; 32])),
    )
    .await?;

    let apeer = base32::encode(base32::Alphabet::Rfc4648Lower { padding: false }, &[0; 32]);
    let bpeer = base32::encode(base32::Alphabet::Rfc4648Lower { padding: false }, &[1; 32]);

    tokio::select! {
        _ = alice.spawn(Arc::new(Metrics::new(apeer.as_str()))) => {}
        _ = bob.spawn(Arc::new(Metrics::new(bpeer.as_str()))) => {}
    }
    Ok(())
}
