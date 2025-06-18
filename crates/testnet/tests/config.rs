use spacejam_testnet::Testnet;

const CONFIG: &str = r#"
[network]
spec = "spacejam/spec/dev/spec.json"

[node.alice]
command = "spacejam"
arch = "polkajam"
data = "alice"
quic = "127.0.0.1:9944"
rpc = "127.0.0.1:9933"
args = []
seed = "0"
env = { RUST_LOG = "debug" }
"#;

#[test]
fn parse_toml() {
    let testnet: Testnet = toml::from_str(CONFIG).unwrap();
    assert_eq!(testnet.node.len(), 1);
    assert_eq!(testnet.node.get("alice").unwrap().command, "spacejam");
}
