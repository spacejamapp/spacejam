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

    let context = alice.context.clone();
    tokio::select! {
        _ = alice.spawn() => {}
        _ = bob.spawn() => {}
        _ = async {
            if *context.read().await > 0 {
                return;
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        } => {}
    }
}
