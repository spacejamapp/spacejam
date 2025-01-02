use spacejam_network::{Client, Config, Server};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use tracing_subscriber::EnvFilter;

fn load_config() -> Config {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut config = Config::try_from(root.join("tests/data/config.toml").as_path())
        .expect("Failed to parse config");
    config.server.der.cert = vec![root.join("tests/data/test.server.cert")];
    config.server.der.key = root.join("tests/data/test.server.pkcs8.pem");
    config.client.der = config.server.der.clone();
    config
}

#[tokio::test]
async fn test_handshake() {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = load_config();
    let server = Server::new(&config).expect("Failed to create server");
    let client = Client::new(&config).expect("Failed to create client");

    let server_task = server.run();
    let client_task = async {
        let timeout = Instant::now();
        loop {
            tokio::time::sleep(Duration::from_millis(1000)).await;

            let response = client
                .request(config.server.addr.into(), b"PING".into())
                .await;

            if response.is_ok() {
                return response;
            }

            if timeout.elapsed() > Duration::from_secs(3) {
                return Err(anyhow::anyhow!("Timeout for 3 seconds: {response:?}",));
            }
        }
    };

    let response = tokio::select!(
        err = server_task => panic!("Server task completed before client task: {err:?}"),
        response = client_task => response,
    )
    .expect("Failed to get response from client task");
    assert_eq!(response, b"OK");
}
