use clap::Parser;
use litep2p::types::multiaddr::Multiaddr;
use spacejam_network::{Config, Network};
use std::str::FromStr;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "0")]
    port: u16,
    #[arg(short, long, default_value = "127.0.0.1")]
    ip: String,
    /// Address to dial
    #[arg(short, long)]
    dial: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let mut config = Config::default();
    if let Some(dial) = args.dial {
        config
            .bootstrap
            .push(Multiaddr::from_str(&dial).expect("invalid address"));
    }

    let mut network = Network::new(config)
        .await
        .expect("failed to create network");
    network.spawn().await;
}
