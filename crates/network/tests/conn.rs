//! Tests for connections.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use network::{Config, Network};
use spacejam_network as network;
use tokio::sync::mpsc;

#[tokio::test]
async fn connections() -> anyhow::Result<()> {
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
        None,
    )
    .await?;

    let bob = Network::new(
        Config {
            address: SocketAddr::new(localhost.into(), 0),
            bootstrap: vec![address],
            genesis: [0; 32],
        },
        brx,
        None,
    )
    .await?;

    Ok(())
}
